"use client"

import { useState } from "react"
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  Legend, 
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell
} from 'recharts'

// Mock data generation
const generateMockData = () => {
  return [
    { name: "OpenAI", requests: 7824, tokens: 1450000, color: "#10a37f" },
    { name: "Anthropic", requests: 3452, tokens: 876000, color: "#b166e9" },
    { name: "Google", requests: 2156, tokens: 498000, color: "#4285f4" },
    { name: "Azure", requests: 1872, tokens: 423000, color: "#0078d4" },
    { name: "Others", requests: 785, tokens: 102000, color: "#888888" }
  ]
}

interface LabelProps {
  cx: number;
  cy: number;
  midAngle: number;
  innerRadius: number;
  outerRadius: number;
  percent: number;
  index: number;
}

export function LlmUsageChart() {
  const [data] = useState(generateMockData())
  const [activeView, setActiveView] = useState("requests")
  
  const RADIAN = Math.PI / 180
  const renderCustomizedLabel = ({ cx, cy, midAngle, innerRadius, outerRadius, percent, index }: LabelProps) => {
    const radius = innerRadius + (outerRadius - innerRadius) * 0.5
    const x = cx + radius * Math.cos(-midAngle * RADIAN)
    const y = cy + radius * Math.sin(-midAngle * RADIAN)
  
    return (
      <text 
        x={x} 
        y={y} 
        fill="white" 
        textAnchor={x > cx ? 'start' : 'end'} 
        dominantBaseline="central"
      >
        {`${(percent * 100).toFixed(0)}%`}
      </text>
    )
  }
  
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
          onClick={() => setActiveView("tokens")}
          className={`px-3 py-1 text-sm rounded-md ${activeView === "tokens" 
            ? "bg-primary text-primary-foreground" 
            : "bg-secondary text-secondary-foreground"}`}
        >
          Tokens
        </button>
        <button 
          onClick={() => setActiveView("distribution")}
          className={`px-3 py-1 text-sm rounded-md ${activeView === "distribution" 
            ? "bg-primary text-primary-foreground" 
            : "bg-secondary text-secondary-foreground"}`}
        >
          Distribution
        </button>
      </div>
      
      <ResponsiveContainer width="100%" height={250}>
        {activeView === "requests" ? (
          <BarChart data={data}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Bar dataKey="requests" fill="#8884d8">
              {data.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={entry.color} />
              ))}
            </Bar>
          </BarChart>
        ) : activeView === "tokens" ? (
          <BarChart data={data}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <Tooltip />
            <Bar dataKey="tokens" fill="#82ca9d">
              {data.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={entry.color} />
              ))}
            </Bar>
          </BarChart>
        ) : (
          <PieChart>
            <Pie
              data={data}
              cx="50%"
              cy="50%"
              labelLine={false}
              label={renderCustomizedLabel}
              outerRadius={100}
              fill="#8884d8"
              dataKey="requests"
            >
              {data.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={entry.color} />
              ))}
            </Pie>
            <Tooltip />
            <Legend />
          </PieChart>
        )}
      </ResponsiveContainer>
    </div>
  )
} 