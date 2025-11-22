// Production Monitoring Package for Sultan L1
//
// Prometheus metrics integration for production monitoring

package monitoring

import (
	"runtime"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	// Block metrics
	BlockHeight = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "sultan_block_height",
		Help: "Current blockchain height",
	})

	BlockProcessingDuration = promauto.NewHistogram(prometheus.HistogramOpts{
		Name:    "sultan_block_processing_duration_seconds",
		Help:    "Time taken to process a block",
		Buckets: prometheus.DefBuckets,
	})

	// Transaction metrics
	TxProcessed = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "sultan_transactions_processed_total",
			Help: "Total number of transactions processed",
		},
		[]string{"status"}, // success, failed
	)

	TxInMempool = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "sultan_mempool_size",
		Help: "Number of transactions in mempool",
	})

	// API metrics
	APIRequests = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "sultan_api_requests_total",
			Help: "Total number of API requests",
		},
		[]string{"endpoint", "method", "status"},
	)

	APILatency = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "sultan_api_latency_seconds",
			Help:    "API request latency",
			Buckets: []float64{.001, .005, .01, .025, .05, .1, .25, .5, 1, 2.5, 5, 10},
		},
		[]string{"endpoint"},
	)

	// FFI bridge metrics
	FFICalls = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "sultan_ffi_calls_total",
			Help: "Total number of FFI calls to Rust core",
		},
		[]string{"function", "status"},
	)

	FFIDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "sultan_ffi_duration_seconds",
			Help:    "Time spent in FFI calls",
			Buckets: []float64{.00001, .00005, .0001, .0005, .001, .005, .01, .05},
		},
		[]string{"function"},
	)

	// System metrics
	GoRoutines = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "sultan_goroutines",
		Help: "Number of goroutines",
	})

	MemoryAlloc = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "sultan_memory_alloc_bytes",
		Help: "Bytes allocated and still in use",
	})

	MemorySys = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "sultan_memory_sys_bytes",
		Help: "Bytes obtained from system",
	})

	// Consensus metrics
	ValidatorPower = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "sultan_validator_power",
			Help: "Validator voting power",
		},
		[]string{"address"},
	)

	ConsensusRound = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "sultan_consensus_round",
		Help: "Current consensus round",
	})

	// IBC metrics
	IBCPackets = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "sultan_ibc_packets_total",
			Help: "Total number of IBC packets",
		},
		[]string{"direction", "status"}, // sent/received, success/failed
	)

	IBCChannels = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "sultan_ibc_channels_active",
		Help: "Number of active IBC channels",
	})
)

// StartSystemMetricsCollector starts background goroutine to collect system metrics
func StartSystemMetricsCollector() {
	go func() {
		ticker := time.NewTicker(10 * time.Second)
		defer ticker.Stop()

		for range ticker.C {
			collectSystemMetrics()
		}
	}()
}

func collectSystemMetrics() {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)

	GoRoutines.Set(float64(runtime.NumGoroutine()))
	MemoryAlloc.Set(float64(m.Alloc))
	MemorySys.Set(float64(m.Sys))
}

// RecordBlockProcessed records a successfully processed block
func RecordBlockProcessed(height int64, duration time.Duration) {
	BlockHeight.Set(float64(height))
	BlockProcessingDuration.Observe(duration.Seconds())
}

// RecordTransaction records a processed transaction
func RecordTransaction(success bool) {
	status := "success"
	if !success {
		status = "failed"
	}
	TxProcessed.WithLabelValues(status).Inc()
}

// RecordAPIRequest records an API request
func RecordAPIRequest(endpoint, method, status string, duration time.Duration) {
	APIRequests.WithLabelValues(endpoint, method, status).Inc()
	APILatency.WithLabelValues(endpoint).Observe(duration.Seconds())
}

// RecordFFICall records an FFI call to Rust core
func RecordFFICall(function string, success bool, duration time.Duration) {
	status := "success"
	if !success {
		status = "failed"
	}
	FFICalls.WithLabelValues(function, status).Inc()
	FFIDuration.WithLabelValues(function).Observe(duration.Seconds())
}

// RecordIBCPacket records an IBC packet transfer
func RecordIBCPacket(direction string, success bool) {
	status := "success"
	if !success {
		status = "failed"
	}
	IBCPackets.WithLabelValues(direction, status).Inc()
}

// UpdateMempoolSize updates the current mempool size
func UpdateMempoolSize(size int) {
	TxInMempool.Set(float64(size))
}

// UpdateValidatorPower updates validator voting power
func UpdateValidatorPower(address string, power int64) {
	ValidatorPower.WithLabelValues(address).Set(float64(power))
}

// UpdateConsensusRound updates the current consensus round
func UpdateConsensusRound(round int32) {
	ConsensusRound.Set(float64(round))
}

// UpdateIBCChannels updates the count of active IBC channels
func UpdateIBCChannels(count int) {
	IBCChannels.Set(float64(count))
}
