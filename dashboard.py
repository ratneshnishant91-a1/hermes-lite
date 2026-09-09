from __future__ import annotations

from fastapi import FastAPI, Request, HTTPException, Depends
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.security.http import HTTPBearer
from pydantic import BaseModel
from typing import Optional
import sqlite3
import json
from datetime import datetime

app = FastAPI(title="Hermes-Lite Dashboard", version="1.5")

# Simple bearer token auth
bearer_scheme = HTTPBearer(auto_error=False)

async def verify_token(token: str = Depends(bearer_scheme)) -> str:
    if not token or token != "dashboard-secret-token":
        raise HTTPException(status_code=401, detail="Unauthorized")
    return token


class ChatRequest(BaseModel):
    message: str
    user_id: str = "default"


class TokenRequest(BaseModel):
    user_id: str
    expires_hours: int = 24


@app.get("/", response_class=HTMLResponse)
async def dashboard():
    return """
<!DOCTYPE html>
<html>
<head>
    <title>Hermes-Lite v1.5 Dashboard</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body class="bg-gray-900 text-white">
    <div class="container mx-auto p-8">
        <h1 class="text-4xl font-bold mb-8">🚀 Hermes-Lite v1.5 Dashboard</h1>
        
        <!-- Stats Grid -->
        <div class="grid grid-cols-4 gap-6 mb-8">
            <div class="bg-gray-800 p-6 rounded-lg">
                <h3 class="text-gray-400">Total Requests</h3>
                <p id="total-requests" class="text-3xl font-bold">-</p>
            </div>
            <div class="bg-gray-800 p-6 rounded-lg">
                <h3 class="text-gray-400">Success Rate</h3>
                <p id="success-rate" class="text-3xl font-bold">-</p>
            </div>
            <div class="bg-gray-800 p-6 rounded-lg">
                <h3 class="text-gray-400">Avg Latency</h3>
                <p id="avg-latency" class="text-3xl font-bold">-</p>
            </div>
            <div class="bg-gray-800 p-6 rounded-lg">
                <h3 class="text-gray-400">Active Sessions</h3>
                <p id="active-sessions" class="text-3xl font-bold">-</p>
            </div>
        </div>

        <!-- Chart -->
        <div class="bg-gray-800 p-6 rounded-lg mb-8">
            <h3 class="text-xl font-bold mb-4">📊 Request History</h3>
            <canvas id="requests-chart" height="80"></canvas>
        </div>

        <!-- Recent Logs -->
        <div class="bg-gray-800 p-6 rounded-lg">
            <h3 class="text-xl font-bold mb-4">📝 Recent Activity</h3>
            <div id="logs-container" class="space-y-2"></div>
        </div>
    </div>

    <script>
        const API_TOKEN = "dashboard-secret-token";
        
        async function fetchMetrics() {
            const res = await fetch('/api/metrics', {
                headers: {'Authorization': `Bearer ${API_TOKEN}`}
            });
            const data = await res.json();
            
            document.getElementById('total-requests').textContent = data.total_requests;
            document.getElementById('success-rate').textContent = `${data.success_rate.toFixed(1)}%`;
            document.getElementById('avg-latency').textContent = `${data.avg_latency_ms.toFixed(0)}ms`;
            document.getElementById('active-sessions').textContent = data.total_requests > 0 ? 'Active' : 'Idle';
        }

        async function fetchLogs() {
            const res = await fetch('/api/logs?limit=10', {
                headers: {'Authorization': `Bearer ${API_TOKEN}`}
            });
            const data = await res.json();
            
            const container = document.getElementById('logs-container');
            container.innerHTML = data.logs.map(log => `
                <div class="bg-gray-700 p-3 rounded">
                    <div class="flex justify-between">
                        <span class="text-sm text-gray-400">${log.timestamp}</span>
                        <span class="text-sm ${log.error ? 'text-red-400' : 'text-green-400'}">
                            ${log.error ? '❌ Error' : '✅ Success'}
                        </span>
                    </div>
                    <p class="mt-2 text-sm">${log.user_message.substring(0, 100)}...</p>
                    <p class="text-xs text-gray-500 mt-1">${log.provider}/${log.model} • ${log.latency_ms.toFixed(0)}ms</p>
                </div>
            `).join('');
        }

        fetchMetrics();
        fetchLogs();
        setInterval(() => {
            fetchMetrics();
            fetchLogs();
        }, 5000);
    </script>
</body>
</html>
"""


@app.get("/api/metrics")
async def get_metrics(token: str = Depends(verify_token)):
    # In production, connect to actual observability
    return {
        "total_requests": 0,
        "successful_requests": 0,
        "failed_requests": 0,
        "success_rate": 0.0,
        "avg_latency_ms": 0.0,
        "total_tokens": 0,
    }


@app.get("/api/logs")
async def get_logs(limit: int = 10, token: str = Depends(verify_token)):
    # In production, fetch from observability
    return {"logs": []}


@app.post("/api/chat")
async def chat(request: ChatRequest, token: str = Depends(verify_token)):
    return {"response": "Dashboard chat not implemented. Use CLI or gateway."}


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="127.0.0.1", port=8080)
