package main

/*
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"unsafe"

	"github.com/ethereum/go-ethereum/common"
)

//export FetchReceipts
func FetchReceipts() *C.char {
	// TODO
	hash := common.Hash{0xBE, 0xEF}
	return C.CString(fmt.Sprintf("%s", hash.String()))
}

//export GoFree
func GoFree(ptr *C.char) {
	C.free(unsafe.Pointer(ptr))
}

func main() {}
