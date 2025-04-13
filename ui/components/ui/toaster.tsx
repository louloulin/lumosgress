"use client"

import {
  Toast,
  ToastClose,
  ToastDescription,
  ToastProvider,
  ToastTitle,
  ToastViewport,
} from "@/components/ui/toast"
import { useToast } from "@/components/ui/use-toast"
import { CheckCircle, AlertCircle, Info, AlertTriangle } from "lucide-react"

export function Toaster() {
  const { toasts } = useToast()

  return (
    <ToastProvider>
      {toasts.map(function ({ id, title, description, action, type, ...props }) {
        return (
          <Toast key={id} {...props} variant={type}>
            <div className="flex gap-3">
              {type === "success" && <CheckCircle className="h-5 w-5 text-green-500" />}
              {type === "destructive" && <AlertCircle className="h-5 w-5 text-red-500" />}
              {type === "warning" && <AlertTriangle className="h-5 w-5 text-amber-500" />}
              {type === "info" && <Info className="h-5 w-5 text-blue-500" />}
              <div className="grid gap-1">
                {title && <ToastTitle>{title}</ToastTitle>}
                {description && (
                  <ToastDescription>{description}</ToastDescription>
                )}
              </div>
            </div>
            {action}
            <ToastClose />
          </Toast>
        )
      })}
      <ToastViewport />
    </ToastProvider>
  )
} 