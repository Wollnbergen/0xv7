package pkg

import (
    "fmt"
    "net"
    "sync"
)

type P2PNetwork struct {
    peers    map[string]*Peer
    listener net.Listener
    mu       sync.RWMutex
}

type Peer struct {
    ID       string
    Address  string
    Connected bool
}

func NewP2PNetwork(port string) (*P2PNetwork, error) {
    listener, err := net.Listen("tcp", ":"+port)
    if err != nil {
        return nil, err
    }
    
    return &P2PNetwork{
        peers:    make(map[string]*Peer),
        listener: listener,
    }, nil
}

func (p *P2PNetwork) Start() {
    go func() {
        for {
            conn, err := p.listener.Accept()
            if err != nil {
                continue
            }
            go p.handleConnection(conn)
        }
    }()
    fmt.Println("P2P Network started on", p.listener.Addr())
}

func (p *P2PNetwork) handleConnection(conn net.Conn) {
    defer conn.Close()
    // Handle peer connection
    peerAddr := conn.RemoteAddr().String()
    
    p.mu.Lock()
    p.peers[peerAddr] = &Peer{
        ID:       peerAddr,
        Address:  peerAddr,
        Connected: true,
    }
    p.mu.Unlock()
    
    fmt.Printf("New peer connected: %s\n", peerAddr)
}
