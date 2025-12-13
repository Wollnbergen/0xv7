#!/bin/bash

echo "☁️ CLOUD DEPLOYMENT OPTIONS FOR SULTAN CHAIN"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

case "${1:-menu}" in
    aws)
        echo "🔶 Deploying to AWS..."
        echo "1. Creating EKS cluster:"
        echo "   eksctl create cluster --name sultan-mainnet --region us-east-1 --nodes 3"
        echo "2. Deploying Sultan Chain:"
        echo "   kubectl apply -f sultan-mainnet/deployments/"
        echo "3. Exposing services:"
        echo "   kubectl expose deployment sultan-mainnet --type=LoadBalancer --port=26657"
        ;;
    
    gcp)
        echo "🔵 Deploying to Google Cloud..."
        echo "1. Creating GKE cluster:"
        echo "   gcloud container clusters create sultan-mainnet --num-nodes=3 --zone=us-central1-a"
        echo "2. Getting credentials:"
        echo "   gcloud container clusters get-credentials sultan-mainnet"
        echo "3. Deploying:"
        echo "   kubectl apply -f sultan-mainnet/deployments/"
        ;;
    
    azure)
        echo "🔷 Deploying to Azure..."
        echo "1. Creating AKS cluster:"
        echo "   az aks create --resource-group sultan-rg --name sultan-mainnet --node-count 3"
        echo "2. Getting credentials:"
        echo "   az aks get-credentials --resource-group sultan-rg --name sultan-mainnet"
        echo "3. Deploying:"
        echo "   kubectl apply -f sultan-mainnet/deployments/"
        ;;
    
    docker)
        echo "🐳 Local Docker Deployment..."
        cd /workspaces/0xv7/sultan-mainnet
        docker build -t sultan-chain:mainnet . 2>/dev/null
        docker run -d -p 26657:26657 -p 26656:26656 --name sultan-mainnet sultan-chain:mainnet
        echo "✅ Docker container started"
        echo "   View logs: docker logs sultan-mainnet"
        ;;
    
    *)
        echo "Select deployment target:"
        echo "  ./DEPLOY_TO_CLOUD.sh aws    - Deploy to AWS"
        echo "  ./DEPLOY_TO_CLOUD.sh gcp    - Deploy to Google Cloud"
        echo "  ./DEPLOY_TO_CLOUD.sh azure  - Deploy to Azure"
        echo "  ./DEPLOY_TO_CLOUD.sh docker - Deploy locally with Docker"
        ;;
esac

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
