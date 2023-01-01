docker stop helibot && docker rm helibot
docker run -d --name helibot --network="host" --restart unless-stopped helibot
