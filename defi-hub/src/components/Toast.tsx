import { useState, useEffect, createContext, useContext, useCallback, ReactNode } from 'react';

type ToastType = 'success' | 'error' | 'info';

interface Toast {
  id: number;
  message: string;
  type: ToastType;
}

interface ToastContextType {
  showToast: (message: string, type?: ToastType) => void;
}

const ToastContext = createContext<ToastContextType | null>(null);

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within ToastProvider');
  }
  return context;
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const showToast = useCallback((message: string, type: ToastType = 'info') => {
    const id = Date.now();
    setToasts((prev) => [...prev, { id, message, type }]);

    // Auto dismiss after 4 seconds
    setTimeout(() => {
      setToasts((prev) => prev.filter((t) => t.id !== id));
    }, 4000);
  }, []);

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}
      <div className="fixed bottom-20 md:bottom-6 left-1/2 -translate-x-1/2 z-[100] flex flex-col gap-2">
        {toasts.map((toast) => (
          <ToastItem key={toast.id} toast={toast} onDismiss={() => setToasts((prev) => prev.filter((t) => t.id !== toast.id))} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

function ToastItem({ toast, onDismiss }: { toast: Toast; onDismiss: () => void }) {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    // Trigger animation
    requestAnimationFrame(() => setVisible(true));
  }, []);

  const icons = {
    success: '✓',
    error: '✕',
    info: 'ℹ',
  };

  const colors = {
    success: 'border-green-500/50 text-green-400',
    error: 'border-red-500/50 text-red-400',
    info: 'border-sultan-gold/50 text-sultan-gold',
  };

  return (
    <div
      className={`
        glass-card flex items-center gap-3 px-4 py-3 min-w-[280px] max-w-[90vw]
        border ${colors[toast.type]}
        transition-all duration-300 ease-out
        ${visible ? 'translate-y-0 opacity-100' : 'translate-y-4 opacity-0'}
      `}
      role="alert"
    >
      <span className={`text-lg ${colors[toast.type]}`}>{icons[toast.type]}</span>
      <span className="text-white flex-1">{toast.message}</span>
      <button
        onClick={onDismiss}
        className="text-white/50 hover:text-white transition-colors"
        aria-label="Dismiss"
      >
        ✕
      </button>
    </div>
  );
}
