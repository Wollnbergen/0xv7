package types

import (
	"context"
	
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"google.golang.org/grpc"
)

// RegisterCodec registers the necessary x/sultan interfaces and concrete types
// on the provided LegacyAmino codec.
func RegisterCodec(cdc *codec.LegacyAmino) {
	cdc.RegisterConcrete(&MsgSend{}, "sultan/MsgSend", nil)
	cdc.RegisterConcrete(&MsgCreateValidator{}, "sultan/MsgCreateValidator", nil)
}

// RegisterInterfaces registers the x/sultan module's interface types
func RegisterInterfaces(registry types.InterfaceRegistry) {
	registry.RegisterImplementations((*sdk.Msg)(nil),
		&MsgSend{},
		&MsgCreateValidator{},
	)
	
	// Service descriptors registered via module registration
}

var (
	amino     = codec.NewLegacyAmino()
	ModuleCdc = codec.NewAminoCodec(amino)
)

func init() {
	RegisterCodec(amino)
	amino.Seal()
}

// Service descriptors for gRPC registration
var _Msg_serviceDesc = grpc.ServiceDesc{
	ServiceName: "sultan.v1.Msg",
	HandlerType: (*MsgServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "Send",
			Handler:    _Msg_Send_Handler,
		},
		{
			MethodName: "CreateValidator",
			Handler:    _Msg_CreateValidator_Handler,
		},
	},
}

func _Msg_Send_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(MsgSend)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(MsgServer).Send(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/sultan.v1.Msg/Send",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(MsgServer).Send(ctx, req.(*MsgSend))
	}
	return interceptor(ctx, in, info, handler)
}

func _Msg_CreateValidator_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(MsgCreateValidator)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(MsgServer).CreateValidator(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/sultan.v1.Msg/CreateValidator",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(MsgServer).CreateValidator(ctx, req.(*MsgCreateValidator))
	}
	return interceptor(ctx, in, info, handler)
}

// RegisterMsgServer registers the msg server
func RegisterMsgServer(s grpc.ServiceRegistrar, srv MsgServer) {
	s.RegisterService(&_Msg_serviceDesc, srv)
}

var _Query_serviceDesc = grpc.ServiceDesc{
	ServiceName: "sultan.v1.Query",
	HandlerType: (*QueryServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "Balance",
			Handler:    _Query_Balance_Handler,
		},
		{
			MethodName: "BlockchainInfo",
			Handler:    _Query_BlockchainInfo_Handler,
		},
	},
}

func _Query_Balance_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(QueryBalanceRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(QueryServer).Balance(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/sultan.v1.Query/Balance",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(QueryServer).Balance(ctx, req.(*QueryBalanceRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _Query_BlockchainInfo_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(QueryBlockchainInfoRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(QueryServer).BlockchainInfo(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/sultan.v1.Query/BlockchainInfo",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(QueryServer).BlockchainInfo(ctx, req.(*QueryBlockchainInfoRequest))
	}
	return interceptor(ctx, in, info, handler)
}

// RegisterQueryServer registers the query server
func RegisterQueryServer(s grpc.ServiceRegistrar, srv QueryServer) {
	s.RegisterService(&_Query_serviceDesc, srv)
}

// NewQueryClient creates a new QueryClient
func NewQueryClient(cc grpc.ClientConnInterface) QueryClient {
	return &queryClient{cc}
}

type queryClient struct {
	cc grpc.ClientConnInterface
}

func (c *queryClient) Balance(ctx context.Context, req *QueryBalanceRequest, opts ...grpc.CallOption) (*QueryBalanceResponse, error) {
	out := new(QueryBalanceResponse)
	err := c.cc.Invoke(ctx, "/sultan.v1.Query/Balance", req, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *queryClient) BlockchainInfo(ctx context.Context, req *QueryBlockchainInfoRequest, opts ...grpc.CallOption) (*QueryBlockchainInfoResponse, error) {
	out := new(QueryBlockchainInfoResponse)
	err := c.cc.Invoke(ctx, "/sultan.v1.Query/BlockchainInfo", req, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// QueryClient interface
type QueryClient interface {
	Balance(ctx context.Context, req *QueryBalanceRequest, opts ...grpc.CallOption) (*QueryBalanceResponse, error)
	BlockchainInfo(ctx context.Context, req *QueryBlockchainInfoRequest, opts ...grpc.CallOption) (*QueryBlockchainInfoResponse, error)
}
