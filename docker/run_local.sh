#! /bin/bash
cargo build --release
./target/release/graph-node \
    --postgres-url "postgresql://admin:123123@localhost:5433/graph-node" \
    --ipfs "localhost:5002" \
    --ethereum-rpc "base:https://base-rpc.publicnode.com" \
    --GRAPH_LOG error