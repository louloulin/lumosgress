"use client"

import { useState } from "react"
import { 
  LineChart, 
  Line, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  Legend, 
  ResponsiveContainer,
  Area,
  AreaChart
} from 'recharts'

// Mock data generation
const generateMockData = () => {
  const data = []
  const now = new Date()
  
  for (let i = 29; i >= 0; i--) {
    const date = new Date(now)
    date.setDate(date.getDate() - i)
    
    data.push({
      date: date.toLocaleDateString("en-US", { month: 'short', day: 'numeric' }),
      requests: Math.floor(Math.random() * 500) + 1000,
      successRate: 99.8 + (Math.random() * 0.2),
      avgResponseTime: Math.floor(Math.random() * 100) + 250
    })
  }
  
  return data
}

export function TrafficChart() {
  const [data] = useState(generateMockData())
  const [activeView, setActiveView] = useState("requests")
  
  return (
    <div className="h-full w-full">
      <div className="mb-4 flex items-center space-x-2">
        <button 
          onClick={() => setActiveView("requests")}
          className={`px-3 py-1 text-sm rounded-md ${activeView === "requests" 
            ? "bg-primary text-primary-foreground" 
            : "bg-secondary text-secondary-foreground"}`}
        >
          Requests
        </button>
        <button 
          onClick={() => setActiveView("responseTimes")}
          className={`px-3 py-1 text-sm rounded-md ${activeView === "responseTimes" 
            ? "bg-primary text-primary-foreground" 
            : "bg-secondary text-secondary-foreground"}`}
        >
          Response Times
        </button>
        <button 
          onClick={() => setActiveView("successRate")}
          className={`px-3 py-1 text-sm rounded-md ${activeView === "successRate" 
            ? "bg-primary text-primary-foreground" 
            : "bg-secondary text-secondary-foreground"}`}
        >
          Success Rate
        </button>
      </div>
      
      <ResponsiveContainer width="100%" height={250}>
        {activeView === "requests" ? (
          <AreaChart data={data}>
            <defs>
              <linearGradient id="colorRequests" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#8884d8" stopOpacity={0.8}/>
                <stop offset="95%" stopColor="#8884d8" stopOpacity={0}/>
              </linearGradient>
            </defs>
            <XAxis dataKey="date" />
            <YAxis />
            <CartesianGrid strokeDasharray="3 3" />
            <Tooltip />
            <Area type="monotone" dataKey="requests" stroke="#8884d8" fillOpacity={1} fill="url(#colorRequests)" />
          </AreaChart>
        ) : activeView === "responseTimes" ? (
          <LineChart data={data}>
            <XAxis dataKey="date" />
            <YAxis />
            <CartesianGrid strokeDasharray="3 3" />
            <Tooltip />
            <Line type="monotone" dataKey="avgResponseTime" stroke="#82ca9d" />
          </LineChart>
        ) : (
          <LineChart data={data}>
            <XAxis dataKey="date" />
            <YAxis domain={[99.5, 100]} />
            <CartesianGrid strokeDasharray="3 3" />
            <Tooltip />
            <Line type="monotone" dataKey="successRate" stroke="#ff7300" />
          </LineChart>
        )}
      </ResponsiveContainer>
    </div>
  )
} 