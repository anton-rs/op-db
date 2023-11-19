package main

/*
#include <stdlib.h>
*/
import "C"
import "unsafe"

// import (
// 	"github.com/ethereum/go-ethereum/core/rawdb"
// )

//export FetchReceipts
func FetchReceipts() *C.char {
	return C.CString("hi from the freezer")
}

//export GoFree
func GoFree(ptr *C.char) {
	C.free(unsafe.Pointer(ptr))
}

func main() {}
