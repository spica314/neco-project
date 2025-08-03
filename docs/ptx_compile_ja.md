# PTX compile

## Functions


### `CUresult cuInit ( unsigned int  Flags )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__INITIALIZE.html

Flagsは0固定
1回だけ必要

### `CUresult cuDeviceGet ( CUdevice* device, int  ordinal )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__DEVICE.html#group__CUDA__DEVICE_1g8bdd1cc7201304b01357b8034f6587cb

deviceが指す先のCUdeviceに初期化して値が入る

GPU0版を使うなら cuDeviceGet(&cu_device, 0)で固定でいい

### `CUresult cuCtxCreate ( CUcontext* pctx, unsigned int  flags, CUdevice dev )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__CTX.html#group__CUDA__CTX_1g65dc0012348bc84810e2103a40d8e2cf

なおdefineされていてcuCtxCreate_v2が本体

CUcontextが初期化されて作られる

cuCtxCreate_v2(&cu_context, 0, cu_device);
の呼び出し固定でよい

### `CUresult cuModuleLoadData ( CUmodule* module, const void* image )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MODULE.html#group__CUDA__MODULE_1g04ce266ce03720f479eab76136b90c0b

CUmoduleが初期化されて作られる。imageはPTXコード(null終端)文字列への参照

### `CUresult cuModuleGetFunction ( CUfunction* hfunc, CUmodule hmod, const char* name )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MODULE.html#group__CUDA__MODULE_1ga52be009b0d4045811b30c965e1cb2cf

CUmoduleから指定した名前の関数を取り出す。CUfunctionに初期化されて入る

### `CUresult cuMemAlloc ( CUdeviceptr* dptr, size_t bytesize )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MEM.html#group__CUDA__MEM_1gb82d2a09844a58dd9e744dc31e8aa467

なおdefineされていてcuMemAlloc_v2が本体
CUdeviceptrが指す先に指定したサイズの領域を確保した領域へのポインタ

### `CUresult cuMemcpyHtoD ( CUdeviceptr dstDevice, const void* srcHost, size_t ByteCount ) `
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MEM.html#group__CUDA__MEM_1g4d32266788c440b0220b1a9ba5795169

なおdefineされていてcuMemcpyHtoD_v2が本体
ホスト上のメモリからdstDeviceが指す領域にコピー

### `CUresult cuLaunchKernel ( CUfunction f, unsigned int  gridDimX, unsigned int  gridDimY, unsigned int  gridDimZ, unsigned int  blockDimX, unsigned int  blockDimY, unsigned int  blockDimZ, unsigned int  sharedMemBytes, CUstream hStream, void** kernelParams, void** extra )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__EXEC.html#group__CUDA__EXEC_1gb8f3dc3031b40da29d5f9a7139e52e15

基本的にsharedMemBytes, hStream, extraは0(NULL)にしておけばOK。
kernelParamsはCUdeviceptrの参照の配列への参照

### `CUresult cuMemcpyDtoH ( void* dstHost, CUdeviceptr srcDevice, size_t ByteCount )`
doc: https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MEM.html#group__CUDA__MEM_1g3480368ee0208a98f75019c9a8450893

なおdefineされていてcuMemcpyDtoH_v2が本体
dstDeviceからホスト上のメモリにコピー
