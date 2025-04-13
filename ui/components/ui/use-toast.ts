export interface ToastProps {
  title?: string;
  description?: string;
  variant?: "default" | "destructive";
}

export interface ToastApi {
  toast: (props: ToastProps) => void;
}

// 简化版的 useToast hook
export function useToast(): ToastApi {
  return {
    toast: (props: ToastProps) => {
      console.log("Toast:", props);
      // 这里可以实现真正的 toast 功能
      // 现在只是打印信息到控制台
    }
  };
} 