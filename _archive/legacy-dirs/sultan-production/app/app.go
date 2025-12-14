package app

const (
    AppName = "sultan"
    DefaultGasPrice = 0 // ZERO GAS FEES!
)

// App represents the Sultan blockchain application
type App struct {
    Name string
    ChainID string
    ZeroGasFees bool
}

// NewApp creates a new Sultan app
func NewApp() *App {
    return &App{
        Name: AppName,
        ChainID: "sultan-1",
        ZeroGasFees: true,
    }
}
