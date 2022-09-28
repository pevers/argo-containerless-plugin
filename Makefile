LOCAL_REGISTRY=mycluster-registry:50959

run-all-dev: build-docker push-image-locally build-plugin push-plugin

.PHONY: build-docker
build-docker:
	docker build -t  containerless .

.PHONY: push-image-locally
push-image-locally:
	docker tag containerless ${LOCAL_REGISTRY}/containerless:local
	docker push ${LOCAL_REGISTRY}/containerless:local

.PHONY: build-plugin
build-plugin:
	argo executor-plugin build argo/

.PHONY: push-plugin
push-plugin:
	kubectl apply -f argo/containerless-executor-plugin-configmap.yaml

.PHONY: submit-test-workflow
submit-test-workflow:
	argo submit -n argo argo/workflow.yaml
