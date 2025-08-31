# Configurable variables
NAMESPACE ?= ccdemo
SECRET_NAME ?= subber-db-credentials
DB_USERNAME ?= appuser
KUBECTL ?= kubectl
OPENSSL ?= openssl
CLIP ?= pbcopy
WEBUI_SERVICE ?= webui
WEBUI_LOCAL_PORT ?= 8080

# Label selector to group resources
LABEL_KEY ?= app.kubernetes.io/part-of
LABEL_VALUE ?= ccdemo

# Wait timeouts
GCP_TIMEOUT ?= 20m
K8S_TIMEOUT ?= 5m

# Optionally provide a password: APPUSER_PASSWORD=...
APPUSER_PASSWORD ?=

.PHONY: create-secret port-forward-webui apply-gcp apply-k8s deploy status delete apply-both demo-pre namespace dbinstance

create-secret:
	@echo "Creating secret '$(SECRET_NAME)' in namespace '$(NAMESPACE)' using README method..."
	@PASS="$(APPUSER_PASSWORD)"; \
	if [ -z "$$PASS" ]; then \
	  PASS=$$(LC_ALL=C tr -dc 'A-Za-z0-9!@#%^*_+=-' </dev/urandom | head -c 24); \
	fi; \
	$(KUBECTL) create secret generic $(SECRET_NAME) -n $(NAMESPACE) \
	  --from-literal=username=$(DB_USERNAME) \
	  --from-literal=password="$$PASS" \
	  -o yaml --dry-run=client | $(KUBECTL) apply -f - >/dev/null
	@echo "Secret '$(SECRET_NAME)' ensured in namespace '$(NAMESPACE)'."

port-forward-webui:
	@echo "Port-forwarding service '$(WEBUI_SERVICE)' in namespace '$(NAMESPACE)' to http://localhost:$(WEBUI_LOCAL_PORT) ..."
	$(KUBECTL) -n $(NAMESPACE) port-forward svc/$(WEBUI_SERVICE) $(WEBUI_LOCAL_PORT):8080

apply-gcp:
	@echo "Applying GCP resources to namespace '$(NAMESPACE)'..."
	$(KUBECTL) -n $(NAMESPACE) apply -f gcp-resources.yaml

apply-k8s:
	@echo "Applying Kubernetes resources to namespace '$(NAMESPACE)'..."
	$(KUBECTL) -n $(NAMESPACE) apply -f k8s-resources.yaml

apply-both: apply-gcp apply-k8s
	@echo "applied both GCP and K8s resource manifests"

# Show status of grouped resources
status:
	@echo "================ Kubernetes resources ================" && \
	$(KUBECTL) -n $(NAMESPACE) get all -l $(LABEL_KEY)=$(LABEL_VALUE) -o wide || true
	@echo "================ Config Connector resources ================" && \
	$(KUBECTL) -n $(NAMESPACE) get pubsubtopic,pubsubsubscription,iamserviceaccount,iampolicymember,sqlinstance,sqldatabase,sqluser -l $(LABEL_KEY)=$(LABEL_VALUE) 2>/dev/null || true

# Clean up resources
delete:
	@echo "Deleting resources in namespace '$(NAMESPACE)'..."
	-$(KUBECTL) -n $(NAMESPACE) delete -f k8s-resources.yaml --ignore-not-found
	-$(KUBECTL) -n $(NAMESPACE) delete -f gcp-resources.yaml --ignore-not-found

namespace:
	$(KUBECTL) -n $(NAMESPACE) apply -f kcc-namespace.yaml

dbinstance:
	$(KUBECTL) -n $(NAMESPACE) apply -f dbinstance.yaml

demo-pre: namespace dbinstance create-secret
