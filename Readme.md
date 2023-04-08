# Scaling Kanban Web App

This project is a prototype of a scaling architecture for a Kanban web app. 

I did this to get some experience with technology I was curious about (mainly Rust Backend, Websockets and Pub/Sub).

# How to run
Simply run `docker compose up --attach backend` and open http://localhost:3500.

# TODO:
Use docker swarm and a load balancer to test out scaling of the app

# Architecture

I wanted to create a solution that would support editing of kanban boards by multiple users across multiple backend instances.

### Backend: Axum (Rust)
Seemed like a good alternative to Actix (it is developed by the Tokio team).

### Database: MongoDB
I decided to use MongoDB as it is easy to set up and to adjust the data layout. 

### Message Service (Pub/Sub): Nats.io
Nats claims to be easy to set up (it was) and I just wanted to try it out.

### Frontend: React (TypeScript)
I created this frontend 2 years ago and reused it for this prototype (it still uses `react-beautiful-dnd`).


### This is how it works:
1. Frontend establishes Websocket connection to Backend for the current board. This will be used to receive updates to the board made by other users  
2. User changes to the board are sent from frontend to backend via `PUT` calls
3. Backend  modifies the boards document in the MongoDB collection, increments a `version` field and returns the result
4. Backend then publishes a message to Pub/Sub containing the new JSON representation of the board
5. Pub/Sub distributes the updated kanban board JSON representation to subscribed backend instances
6. Subscribed backend instances notify connected frontends about change to the kanban board via the Websocket connections of step 1
7. Frontend retrieves updated board state and updates its state if the `version` of the received board is newer
