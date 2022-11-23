package container

import (
	"context"
	"log"
	"os"

	pb "cros.local/bazel/ebuild/private/common/hostcontainercomms/host_service"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func NewClient() (pb.HostServiceClient, *grpc.ClientConn, error) {
	address, err := os.ReadFile("/helpers/server_address")
	if err != nil {
		return nil, nil, err
	}

	var opts []grpc.DialOption
	opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))

	conn, err := grpc.Dial(string(address), opts...)
	if err != nil {
		return nil, nil, err
	}
	client := pb.NewHostServiceClient(conn)
	return client, conn, nil
}

func CreateExecuteAsRootRequest(name string, args ...string) (req *pb.ExecuteAsRootRequest, err error) {
	cwd, err := os.Getwd()
	if err != nil {
		return nil, err
	}
	req = &pb.ExecuteAsRootRequest{
		Name: name,
		Args: args,
		Dir:  cwd,
	}
	return req, err
}

// ExecuteAsRoot executes a command transparently as if it was running inside
// the container.
func ExecuteAsRoot(ctx context.Context, req *pb.ExecuteAsRootRequest) (*pb.ExecuteAsRootResponse, error) {
	cl, conn, err := NewClient()
	if err != nil {
		return nil, err
	}
	defer conn.Close()

	resp, err := cl.ExecuteAsRoot(ctx, req)
	if err != nil {
		return nil, err
	}
	if _, err := os.Stdout.Write(resp.Stdout); err != nil {
		return nil, err
	}
	if _, err := os.Stderr.Write(resp.Stderr); err != nil {
		return nil, err
	}
	return resp, nil
}

// RootExec acts like the syscall exec, but executes the command as real root in
// the namespace.
func RootExec(ctx context.Context, name string, args ...string) {
	req, err := CreateExecuteAsRootRequest(name, args...)
	if err != nil {
		log.Fatal(err)
	}
	resp, err := ExecuteAsRoot(ctx, req)
	if err != nil {
		log.Fatal(err)
	}
	os.Exit(int(resp.ExitCode))
}
