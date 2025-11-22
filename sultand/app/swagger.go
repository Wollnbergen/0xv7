package app

import (
	"embed"
	"io/fs"
	"net/http"
	
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/gorilla/mux"
)

//go:embed swagger-ui
var swaggerUI embed.FS

// RegisterSwaggerAPI registers Swagger/OpenAPI documentation with API Server
// Serves the Swagger UI at /swagger/ for interactive API exploration
func RegisterSwaggerAPI(_ client.Context, rtr *mux.Router) {
	// Get the embedded swagger-ui directory
	swaggerFS, err := fs.Sub(swaggerUI, "swagger-ui")
	if err != nil {
		// If swagger-ui doesn't exist, register a fallback handler
		rtr.PathPrefix("/swagger/").HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusOK)
			w.Write([]byte(`{"info":"Swagger UI not embedded. API docs available at /cosmos/tx/v1beta1/txs, /cosmos/bank/v1beta1/balances, /ibc/core/client/v1/client_states"}`))
		})
		return
	}
	
	// Serve the embedded Swagger UI
	staticServer := http.FileServer(http.FS(swaggerFS))
	rtr.PathPrefix("/swagger/").Handler(http.StripPrefix("/swagger/", staticServer))
}
