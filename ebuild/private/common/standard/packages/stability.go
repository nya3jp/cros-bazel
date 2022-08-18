package packages

type Stability string

const (
	StabilityStable  Stability = "stable"
	StabilityTesting Stability = "testing"
	StabilityBroken  Stability = "broken"
)
