import { useState, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { useTipsStore, type TipId } from '../../stores/tipsStore';

type TipPosition = 'top' | 'bottom' | 'left' | 'right';

interface TipProps {
  tipId: TipId;
  targetRef: React.RefObject<HTMLElement>;
  message: string;
  position?: TipPosition;
  showCondition?: boolean;  // Additional condition beyond "not shown before"
  delayMs?: number;         // Delay before showing (default: 2500)
  autoDismissMs?: number;   // Auto-dismiss after this many ms (default: 8000)
  icon?: 'lightbulb' | 'info' | 'sparkle';
}

export function Tip({
  tipId,
  targetRef,
  message,
  position = 'right',
  showCondition = true,
  delayMs = 2500,
  autoDismissMs = 8000,
  icon = 'lightbulb',
}: TipProps) {
  const [coords, setCoords] = useState({ top: 0, left: 0 });
  const [isVisible, setIsVisible] = useState(false);
  const [isDismissing, setIsDismissing] = useState(false);
  const [targetReady, setTargetReady] = useState(false);
  const tipRef = useRef<HTMLDivElement>(null);

  const hasTipBeenShown = useTipsStore((s) => s.hasTipBeenShown(tipId));
  const markTipShown = useTipsStore((s) => s.markTipShown);

  // Check if target ref is ready (poll briefly on mount)
  useEffect(() => {
    if (targetRef.current) {
      setTargetReady(true);
      return;
    }
    // Poll for target element if not immediately available
    const checkInterval = setInterval(() => {
      if (targetRef.current) {
        setTargetReady(true);
        clearInterval(checkInterval);
      }
    }, 100);
    // Stop checking after 2 seconds
    const timeout = setTimeout(() => clearInterval(checkInterval), 2000);
    return () => {
      clearInterval(checkInterval);
      clearTimeout(timeout);
    };
  }, [targetRef]);

  // Calculate position based on target element
  useEffect(() => {
    if (!targetReady || !targetRef.current || hasTipBeenShown || !showCondition) return;

    const updatePosition = () => {
      const rect = targetRef.current?.getBoundingClientRect();
      if (!rect) return;

      const tipWidth = 240;  // Approximate tip width
      const tipHeight = 60;  // Approximate tip height
      const gap = 12;

      let top = 0;
      let left = 0;

      switch (position) {
        case 'top':
          top = rect.top - tipHeight - gap;
          left = rect.left + rect.width / 2 - tipWidth / 2;
          break;
        case 'bottom':
          top = rect.bottom + gap;
          left = rect.left + rect.width / 2 - tipWidth / 2;
          break;
        case 'left':
          top = rect.top + rect.height / 2 - tipHeight / 2;
          left = rect.left - tipWidth - gap;
          break;
        case 'right':
        default:
          top = rect.top + rect.height / 2 - tipHeight / 2;
          left = rect.right + gap;
          break;
      }

      // Keep within viewport bounds
      left = Math.max(8, Math.min(left, window.innerWidth - tipWidth - 8));
      top = Math.max(8, Math.min(top, window.innerHeight - tipHeight - 8));

      setCoords({ top, left });
    };

    updatePosition();
    // Delay before showing the tip
    const showTimer = setTimeout(() => {
      updatePosition();  // Recalculate in case things moved
      setIsVisible(true);
    }, delayMs);

    window.addEventListener('resize', updatePosition);
    return () => {
      clearTimeout(showTimer);
      window.removeEventListener('resize', updatePosition);
    };
  }, [targetRef, targetReady, position, hasTipBeenShown, showCondition, delayMs]);

  // Reset visibility when showCondition becomes false
  useEffect(() => {
    if (!showCondition && isVisible) {
      setIsVisible(false);
      setIsDismissing(false);
    }
  }, [showCondition, isVisible]);

  // Auto-dismiss timer
  useEffect(() => {
    if (!isVisible || hasTipBeenShown) return;

    const timer = setTimeout(() => {
      handleDismiss();
    }, autoDismissMs);

    return () => clearTimeout(timer);
  }, [isVisible, autoDismissMs, hasTipBeenShown]);

  const handleDismiss = () => {
    setIsDismissing(true);
    setTimeout(() => {
      markTipShown(tipId);
      setIsVisible(false);
    }, 200);
  };

  // Don't render if already shown or condition not met
  if (hasTipBeenShown || !showCondition || !isVisible) {
    return null;
  }

  const getArrowClasses = () => {
    const base = 'absolute w-0 h-0 border-solid';
    switch (position) {
      case 'top':
        return `${base} bottom-[-6px] left-1/2 -translate-x-1/2 border-l-[6px] border-l-transparent border-r-[6px] border-r-transparent border-t-[6px] border-t-accent`;
      case 'bottom':
        return `${base} top-[-6px] left-1/2 -translate-x-1/2 border-l-[6px] border-l-transparent border-r-[6px] border-r-transparent border-b-[6px] border-b-accent`;
      case 'left':
        return `${base} right-[-6px] top-1/2 -translate-y-1/2 border-t-[6px] border-t-transparent border-b-[6px] border-b-transparent border-l-[6px] border-l-accent`;
      case 'right':
      default:
        return `${base} left-[-6px] top-1/2 -translate-y-1/2 border-t-[6px] border-t-transparent border-b-[6px] border-b-transparent border-r-[6px] border-r-accent`;
    }
  };

  const renderIcon = () => {
    switch (icon) {
      case 'info':
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
      case 'sparkle':
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
          </svg>
        );
      case 'lightbulb':
      default:
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
          </svg>
        );
    }
  };

  return createPortal(
    <div
      ref={tipRef}
      onClick={handleDismiss}
      className={`fixed z-[10000] cursor-pointer transition-all duration-200 ${
        isDismissing ? 'opacity-0 scale-95' : 'animate-tip-bounce'
      }`}
      style={{
        top: coords.top,
        left: coords.left,
      }}
    >
      {/* Arrow */}
      <div className={getArrowClasses()} />

      {/* Tip content */}
      <div className="flex items-start gap-2 px-3 py-2.5 bg-accent text-white rounded-lg shadow-lg shadow-accent/25 max-w-[240px]">
        <span className="flex-shrink-0 mt-0.5">{renderIcon()}</span>
        <div>
          <p className="text-sm font-medium leading-snug">{message}</p>
          <p className="text-[10px] opacity-75 mt-1">Click to dismiss</p>
        </div>
      </div>
    </div>,
    document.body
  );
}
