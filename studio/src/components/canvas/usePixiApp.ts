import { useEffect, useRef } from 'react';
import { Application } from 'pixi.js';

export function usePixiApp(containerRef: React.RefObject<HTMLDivElement | null>) {
  const appRef = useRef<Application | null>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    let cancelled = false;
    const app = new Application();
    appRef.current = app;

    app
      .init({
        background: 0x0a0e14,
        resizeTo: container,
        antialias: true,
        resolution: window.devicePixelRatio || 1,
        autoDensity: true,
      })
      .then(() => {
        // If cleanup ran before init resolved, destroy this instance immediately
        if (cancelled) {
          app.destroy(true, { children: true });
          return;
        }
        if (!container.contains(app.canvas)) {
          container.appendChild(app.canvas);
        }
      })
      .catch(console.error);

    return () => {
      cancelled = true;
      // Only destroy if init has already settled; otherwise the then() above handles it
      if (app.renderer) {
        app.destroy(true, { children: true });
      }
      appRef.current = null;
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // intentionally empty — create once per mount

  return appRef;
}
