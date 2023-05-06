docker build \
	--build-arg "http_proxy=http://172.17.128.1:7890" \
	--build-arg "https_proxy=http://172.17.128.1:7890" \
	--build-arg "NO_PROXY=localhost,127.0.0.1,.example.com" \
	-f FirstStage.Dockerfile \
	-t eightfish-m2-build . 
