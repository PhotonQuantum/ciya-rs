RUSTFLAGS = "-C link-arg=-Wl,-rpath,libs "
ONNXRUNTIME_NAME = onnxruntime-linux-x64-1.8.1
ONNXRUNTIME_URL = "https://github.com/microsoft/onnxruntime/releases/download/v1.8.1/${ONNXRUNTIME_NAME}.tgz"
ONNXRUNTIME_SO_PATH = lib/libonnxruntime.so.1.8.1
OPENCV_MODULES = "core,objdetect"

all: cli bot copy-ort

cli: download-ort
	mkdir -p dist
	OPENCV_MODULE_WHITELIST=${OPENCV_MODULES} ORT_STRATEGY=system ORT_LIB_LOCATION=_build/${ONNXRUNTIME_NAME}/ RUSTFLAGS=${RUSTFLAGS} cargo build --bin ciya_cli --release
	cp target/release/ciya_cli dist/

bot: download-ort
	mkdir -p dist
	OPENCV_MODULE_WHITELIST=${OPENCV_MODULES} ORT_STRATEGY=system ORT_LIB_LOCATION=_build/${ONNXRUNTIME_NAME}/ RUSTFLAGS=${RUSTFLAGS} cargo build --bin ciya_bot --release
	cp target/release/ciya_bot dist/

download-ort:
	mkdir -p _build
	wget -N ${ONNXRUNTIME_URL} -P _build
	tar xzvf _build/${ONNXRUNTIME_NAME}.tgz -C _build/

copy-ort: download-ort
	mkdir -p dist/libs
	cp _build/${ONNXRUNTIME_NAME}/${ONNXRUNTIME_SO_PATH} dist/libs/

clean: clean-build clean-dist

clean-build:
	rm -rf _build
	cargo clean

clean-dist:
	rm -rf dist
