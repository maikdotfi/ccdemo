# Config Connector Demo

**NOTE** there's a placeholder for the GCP project id in the manifests: `GCP_PROJECT_ID`. Replace this with `sed` for example in the files with your project ID. Same goes for the `PROJECT_ID` environment variable seen in the README. How to do this is left as an exercise to the reader.

## Config Connector Setup

This section documents the steps to install and configure the Google Cloud Config Connector.

### Prerequisites

* `gcloud` CLI is configured with a project.

Set common vars:

```bash
export PROJECT_ID="GCP_PROJECT_ID"
export REGION="europe-north1"
export ZONE="europe-north1-b"
export CLUSTER_NAME="static-zonal-1"
export SERVICE_ACCOUNT=cnrm-system
```


Boot up a GKE cluster (single zone, cheap and static):

```bash
gcloud container clusters create "$CLUSTER_NAME" \
    --zone "$ZONE" \
    --workload-pool="$PROJECT_ID.svc.id.goog" \
    --release-channel=regular \
    --machine-type=e2-standard-2 \
    --num-nodes=2 \
    --scopes=gke-default \
    --project "$PROJECT_ID"
```

```bash
gcloud container clusters get-credentials "$CLUSTER_NAME" --zone "$ZONE" --project "$PROJECT_ID"
gcloud container clusters describe "$CLUSTER_NAME" --zone "$ZONE" --format='value(workloadIdentityConfig.workloadPool)'
```

### Operator Installation

```bash
gcloud storage cp gs://configconnector-operator/latest/release-bundle.tar.gz release-bundle.tar.gz
tar -xzvf release-bundle.tar.gz
kubectl apply -f operator-system/configconnector-operator.yaml
```

```bash
gcloud iam service-accounts create $SERVICE_ACCOUNT

# give the service account GCP permissions
gcloud projects add-iam-policy-binding $PROJECT_ID \
  --member="serviceAccount:$SERVICE_ACCOUNT@$PROJECT_ID.iam.gserviceaccount.com" \
  --role="roles/owner"

# bind the service account to kubernetes service account, lol same same different
gcloud iam service-accounts add-iam-policy-binding \
$SERVICE_ACCOUNT@$PROJECT_ID.iam.gserviceaccount.com \
  --member="serviceAccount:$PROJECT_ID.svc.id.goog[cnrm-system/cnrm-controller-manager]" \
  --role="roles/iam.workloadIdentityUser"

echo "  googleServiceAccount: \"$SERVICE_ACCOUNT@$PROJECT_ID.iam.gserviceaccount.com\"" >> config-connector.yaml
kubectl apply -f config-connector.yaml
```

```bash
kubectl describe configconnector
kubectl get pods -n cnrm-system -w
```

### Create KCC-enabled Namespace

Create and apply the namespace that will host your KCC-managed resources. Apply this first so later resources can target it if desired.

```bash
kubectl apply -f kcc-namespace.yaml
```

### Enable Required GCP APIs

These services must be enabled once per project for the manifests to reconcile:

```bash
gcloud services enable \
  sqladmin.googleapis.com \
  pubsub.googleapis.com \
  iam.googleapis.com \
  iamcredentials.googleapis.com \
  cloudresourcemanager.googleapis.com \
  --project $PROJECT_ID
```

## Cloud SQL for subber

The manifests define a Postgres instance `ccdemo-postgres`, a database `wordsdb`, and a SQL user `appuser` whose password is read from a Kubernetes Secret. For security, create the Secret dynamically rather than committing it to Git:

```bash
make create-secret
```

Apply order suggestion:

```bash
# 1) Ensure APIs are enabled (once per project)
# 2) Apply GCP resources (Pub/Sub, IAM, Cloud SQL)
kubectl apply -f gcp-resources.yaml

# 3) Wait for the SQLInstance to be ready
kubectl get sqlinstances -w
```

## Demo

In demo the Config Connector instance is running, the namespace is created and the DB password Secret is created.

There's a target for the pre-existing stuff:

```bash
make demo-pre
```

> **NOTE:** make sure to select the namespace because I didn't put it everywhere, lol

Apply gcp+k8s resources:

```bash
kubectl apply -f gcp-resources.yaml
kubectl apply -f k8s-resources.yaml
```

Look at the stuff, look!

```bash
make status
```

Port forward:

```bash
kubectl -n ccdemo port-forward svc/webui 9000:8080
```

if you want to look at the db (in GCP console for example), get the password:

```bash
kubectl get secret subber-db-credentials -n ccdemo -o jsonpath='{.data.password}' | base64 -d | pbcopy
```

