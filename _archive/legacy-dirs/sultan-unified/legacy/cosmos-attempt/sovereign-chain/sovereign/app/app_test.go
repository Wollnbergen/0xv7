package app

import "testing"

func TestBasic(t *testing.T) {
    if 1+1 != 2 {
        t.Error("Math is broken")
    }
}
