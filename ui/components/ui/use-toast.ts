"use client"

import * as React from "react"

import type {
  ToastActionElement,
} from "@/components/ui/toast"

const TOAST_LIMIT = 5

type ToastType = "default" | "destructive" | "success" | "warning" | "info"

type ToastState = {
  id: string
  title?: React.ReactNode
  description?: React.ReactNode
  action?: ToastActionElement
  type?: ToastType
  duration?: number
  open: boolean
}

const ADD_TOAST = "ADD_TOAST"
const UPDATE_TOAST = "UPDATE_TOAST"
const DISMISS_TOAST = "DISMISS_TOAST"
const REMOVE_TOAST = "REMOVE_TOAST"

type Action =
  | {
      type: typeof ADD_TOAST
      toast: Omit<ToastState, "id" | "open">
    }
  | {
      type: typeof UPDATE_TOAST
      toast: Partial<ToastState> & { id: string }
    }
  | {
      type: typeof DISMISS_TOAST
      toastId?: string
    }
  | {
      type: typeof REMOVE_TOAST
      toastId?: string
    }

let count = 0

function genId() {
  count = (count + 1) % Number.MAX_SAFE_INTEGER
  return count.toString()
}

interface State {
  toasts: ToastState[]
}

const reducer = (state: State, action: Action): State => {
  switch (action.type) {
    case ADD_TOAST:
      return {
        ...state,
        toasts: [
          ...state.toasts,
          {
            id: genId(),
            open: true,
            ...action.toast,
          },
        ].slice(0, TOAST_LIMIT),
      }

    case UPDATE_TOAST:
      return {
        ...state,
        toasts: state.toasts.map((t) =>
          t.id === action.toast.id
            ? { ...t, ...action.toast }
            : t
        ),
      }

    case DISMISS_TOAST: {
      const { toastId } = action

      // dismiss all toasts
      if (toastId === undefined) {
        return {
          ...state,
          toasts: state.toasts.map((t) => ({
            ...t,
            open: false,
          })),
        }
      }

      // dismiss single toast
      return {
        ...state,
        toasts: state.toasts.map((t) =>
          t.id === toastId
            ? {
                ...t,
                open: false,
              }
            : t
        ),
      }
    }
    case REMOVE_TOAST: {
      const { toastId } = action

      if (toastId === undefined) {
        return {
          ...state,
          toasts: [],
        }
      }

      return {
        ...state,
        toasts: state.toasts.filter((t) => t.id !== toastId),
      }
    }
  }
}

const listeners: Array<(state: State) => void> = []

let memoryState: State = { toasts: [] }

function dispatch(action: Action) {
  memoryState = reducer(memoryState, action)
  listeners.forEach((listener) => {
    listener(memoryState)
  })
}

interface ToastOptions extends Omit<ToastState, "id" | "open"> {
  id?: string
}

function toast(options: ToastOptions) {
  const id = options.id || genId()

  const update = (props: ToastState) =>
    dispatch({
      type: UPDATE_TOAST,
      toast: { ...props, id },
    })

  const dismiss = () => dispatch({ type: DISMISS_TOAST, toastId: id })

  dispatch({
    type: ADD_TOAST,
    toast: {
      ...options,
      duration: options.duration || 5000,
    },
  })

  return {
    id,
    dismiss,
    update,
  }
}

function useToast() {
  const [state, setState] = React.useState<State>(memoryState)

  React.useEffect(() => {
    listeners.push(setState)
    return () => {
      const index = listeners.indexOf(setState)
      if (index > -1) {
        listeners.splice(index, 1)
      }
    }
  }, [state])

  return {
    ...state,
    toast,
    dismiss: (toastId?: string) => dispatch({ type: DISMISS_TOAST, toastId }),
    success: (props: Omit<ToastOptions, "type">) => toast({ ...props, type: "success" }),
    error: (props: Omit<ToastOptions, "type">) => toast({ ...props, type: "destructive" }),
    warning: (props: Omit<ToastOptions, "type">) => toast({ ...props, type: "warning" }),
    info: (props: Omit<ToastOptions, "type">) => toast({ ...props, type: "info" }),
  }
}

export { useToast, toast } 