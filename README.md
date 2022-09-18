# argo-containerless-plugin

Plugin to deploy Python/Node packages directly from a git repository as Argo Workflow step. 

## Example

This will run this specific Python package from the `main` branch directly as Argo Workflow step.

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Workflow
metadata:
  generateName: containerless-
spec:
  entrypoint: main
  templates:
    - name: main
      plugin:
        containerless:
          # Python >= 3.7 is supported
          runtime: python-3.10
          repoURL: https://github.com/pevers/example-python-step.git
          targetRevision: main
          args:
            - "src/example.py"
            - "--some"
            - "parameter"
```

## Supports

- [x] Python modules that can be installed via Poetry (pyproject.toml)
- [x] Python 3.7, 3.8, 3.9, 3.10
- [ ] Other Python build tools (pipenv, conda)
- [ ] Node

## How it works

The Dockerfile creates an environment having multiple Python versions. We use Poetry to create a new virtual environment per module. Poetry is also used to install all project dependencies via a `pyproject.toml` file.

## Compile and test

This requires an environment with the following tools installed.

- Python
- Git
- Poetry
- PyEnv

```rust
cargo build
RUST_LOG=info ARGO_TOKEN_PATH=test/token cargo test
```

## Deploy in a local Kubernetes cluster

The Makefile assumes a local cluster created with a local registry. For example.

```console
k3d cluster create mycluster --registry-create mycluster-registr
```

Install Argo following [this](https://argoproj.github.io/argo-workflows/quick-start/) tutorial.

Then use the Makefile to deploy the plugin.

```console
make run-all-dev
```

Submit an example workflow by running.

```console
argo submit -n argo/workflow.yaml
```

## Roadmap
- [ ] Semantics should be the same as the Argo "script" step, maybe we should support the `ScriptTemplate` parameter?
- [ ] Make sure all errors are propagated correctly and the user has enough information when something fails
- [ ] Make sure that errors are thrown for incorrect Python scripts
- [ ] Cache modules by mounting a PVC?
- [ ] Support for Node modules
