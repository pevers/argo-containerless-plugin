<!-- This is an auto-generated file. DO NOT EDIT -->
# containerless

* Needs: 
* Image: mycluster-registry:49287/containerless:local



Install:

    kubectl apply -f containerless-executor-plugin-configmap.yaml

Uninstall:
	
    kubectl delete cm containerless-executor-plugin 
