RUSTFLAGS = "-C link-arg=-Wl,-rpath,libs "
ONNXRUNTIME_NAME = onnxruntime-linux-x64-1.6.0
ONNXRUNTIME_URL = "https://github.com/microsoft/onnxruntime/releases/download/v1.6.0/${ONNXRUNTIME_NAME}.tgz"
ONNXRUNTIME_SO_PATH = lib/libonnxruntime.so.1.6.0

cli:
	mkdir -p _build
	mkdir -p dist/ciya-cli/libs
	wget -N ${ONNXRUNTIME_URL} -P _build
	tar xzvf _build/${ONNXRUNTIME_NAME}.tgz -C _build/
	ORT_STRATEGY=system ORT_LIB_LOCATION=_build/${ONNXRUNTIME_NAME}/ RUSTFLAGS=${RUSTFLAGS} cargo build --release
	cp target/release/ciya_cli dist/ciya-cli/
	cp _build/${ONNXRUNTIME_NAME}/${ONNXRUNTIME_SO_PATH} dist/ciya-cli/libs/

clean:
	rm -rf _build
	cargo clean