package host

import (
	"bytes"
	"context"
	"log"
	"net"
	"os"
	"os/exec"
	"strconv"
	"strings"

	pb "cros.local/bazel/ebuild/private/common/hostcontainercomms/host_service"
	"google.golang.org/grpc"
)

const sigKillExitCode = 137

type Server struct {
	pb.UnimplementedHostServiceServer

	pidFile    string
	listener   net.Listener
	grpcServer *grpc.Server
}

func (s *Server) ExecuteAsRoot(ctx context.Context, req *pb.ExecuteAsRootRequest) (*pb.ExecuteAsRootResponse, error) {
	contents, err := os.ReadFile(s.pidFile)
	if err != nil {
		return nil, err
	}
	pid, err := strconv.Atoi(string(contents))
	if err != nil {
		return nil, err
	}

	// Mount all namespaces except the user namespace. This ensures that we still
	// actually have root privileges, but paths such as /host/mnt/source/... will
	// still work.
	cmd := exec.Command("/usr/bin/sudo", append([]string{"nsenter", "--target", strconv.Itoa(pid), "--mount", "--ipc", "--pid", "--", req.Name}, req.Args...)...)

	resp := &pb.ExecuteAsRootResponse{}
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	log.Printf("Executing on host machine: %s", strings.Join(cmd.Args, " "))
	if err := cmd.Run(); err != nil {
		log.Printf("Command execution failed: %v", err)
	}
	resp.Stdout = stdout.Bytes()
	resp.Stderr = stderr.Bytes()
	resp.ExitCode = int32(cmd.ProcessState.ExitCode())
	if len(resp.Stdout) != 0 {
		log.Printf("Stdout: %s", string(resp.Stdout))
	}
	if len(resp.Stderr) != 0 {
		log.Printf("Stderr: %s", string(resp.Stderr))
	}
	log.Printf("Exit code: %d", resp.ExitCode)
	// -1 indicates that either the process was still running or was killed by a
	// signal. Since we used cmd.Run(), the process is no longer running.
	if resp.ExitCode == -1 {
		resp.ExitCode = sigKillExitCode
	}
	return resp, nil
}

func StartServer(ctx context.Context, pidFile string) (*Server, error) {
	lis, err := net.Listen("tcp", ":0")
	if err != nil {
		return nil, err
	}
	var opts []grpc.ServerOption
	// TODO: Consider using grpc credentials in the future.
	grpcServer := grpc.NewServer(opts...)
	server := &Server{listener: lis, grpcServer: grpcServer, pidFile: pidFile}
	pb.RegisterHostServiceServer(grpcServer, server)
	go func() {
		if err := grpcServer.Serve(lis); err != nil {
			log.Printf("GRPC while to serve: %v", err)
		}
	}()
	return server, nil
}

func (s *Server) Close() {
	s.grpcServer.Stop()
}

func (s *Server) Port() int {
	return s.listener.Addr().(*net.TCPAddr).Port
}
