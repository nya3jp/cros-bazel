package makevars

import (
	"fmt"
	"io"
	"sort"
	"strings"

	"github.com/alessio/shellescape"
)

type Vars map[string]string

func (v Vars) Copy() Vars {
	u := make(Vars)
	for key, value := range v {
		u[key] = value
	}
	return u
}

func (v Vars) Environ() []string {
	names := make([]string, 0, len(v))
	for name := range v {
		names = append(names, name)
	}
	sort.Strings(names)

	env := make([]string, 0, len(v))
	for _, name := range names {
		env = append(env, fmt.Sprintf("%s=%s", name, v[name]))
	}
	return env
}

func (v Vars) Dump(w io.Writer) {
	names := make([]string, 0, len(v))
	for name := range v {
		names = append(names, name)
	}
	sort.Strings(names)

	for _, name := range names {
		fmt.Fprintf(w, "%s=%s\n", shellescape.Quote(name), shellescape.Quote(v[name]))
	}
}

func (v Vars) GetAsList(key string) []string {
	return strings.Fields(v[key])
}

func (v Vars) GetAsSet(key string) map[string]struct{} {
	set := make(map[string]struct{})
	for _, e := range v.GetAsList(key) {
		set[e] = struct{}{}
	}
	return set
}

func (v Vars) ComputeUse() map[string]struct{} {
	// TODO: Ensure this computation is correct.
	uses := v.GetAsSet("USE")
	iuses := v.GetAsSet("IUSE")

	computed := make(map[string]struct{})
	for u := range uses {
		if !strings.HasPrefix(u, "-") {
			computed[u] = struct{}{}
		}
	}
	for iuse := range iuses {
		if strings.HasPrefix(iuse, "+") {
			u := strings.TrimPrefix(iuse, "+")
			if _, ok := uses["-"+u]; !ok {
				computed[u] = struct{}{}
			}
		}
	}

	return computed
}

func Merge(varsList ...Vars) Vars {
	merged := make(Vars)
	for _, vars := range varsList {
		// TODO: Treat variables mentioned in USE_EXPAND and its family as incremental.
		for name := range vars {
			merged[name] = mergeVar(name, merged[name], vars[name])
		}
	}
	return merged
}

func mergeVar(name string, oldValue string, newValue string) string {
	if !isIncrementalVar(name) {
		return newValue
	}
	return mergeIncrementalVar(oldValue, newValue)
}

func mergeIncrementalVar(oldValue string, newValue string) string {
	mergedTokenSet := parseTokenSet(oldValue)
	updateTokenSet := parseTokenSet(newValue)

	const removeAll = "-*"
	if _, ok := updateTokenSet[removeAll]; ok {
		mergedTokenSet = make(map[string]struct{})
		delete(updateTokenSet, removeAll)
	}

	for token := range updateTokenSet {
		if strings.HasPrefix(token, "-") {
			delete(mergedTokenSet, token[1:])
		} else {
			mergedTokenSet[token] = struct{}{}
		}
	}

	var mergedTokens []string
	for token := range mergedTokenSet {
		mergedTokens = append(mergedTokens, token)
	}
	sort.Strings(mergedTokens)
	return strings.Join(mergedTokens, " ")
}

func parseTokenSet(s string) map[string]struct{} {
	tokenSet := make(map[string]struct{})
	for _, token := range strings.Fields(s) {
		tokenSet[token] = struct{}{}
	}
	return tokenSet
}

var incrementalVarNames = map[string]struct{}{
	"USE":                   {},
	"USE_EXPAND":            {},
	"USE_EXPAND_HIDDEN":     {},
	"CONFIG_PROTECT":        {},
	"CONFIG_PROTECT_MASK":   {},
	"IUSE_IMPLICIT":         {},
	"USE_EXPAND_IMPLICIT":   {},
	"USE_EXPAND_UNPREFIXED": {},
	"ENV_UNSET":             {},
	// USE_EXPAND_VALUES_* are handled separately.
}

func isIncrementalVar(name string) bool {
	if _, ok := incrementalVarNames[name]; ok {
		return true
	}
	if strings.HasPrefix(name, "USE_EXPAND_VALUES_") {
		return true
	}
	return false
}
