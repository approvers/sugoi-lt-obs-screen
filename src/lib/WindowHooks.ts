import { useEffect, useState } from "react";

type WindowDimensions = {
  width: number;
  height: number;
};

export const useWindowDimensions = (): WindowDimensions => {
  const getWindowDimensions = () => {
    const { innerWidth: width, innerHeight: height } = window;
    return {
      width,
      height,
    };
  };

  const [windowDimensions, setWindowDimensions] = useState(
    getWindowDimensions(),
  );
  useEffect(() => {
    const onResize = () => {
      setWindowDimensions(getWindowDimensions());
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, []);
  return windowDimensions;
};

// https://ryotarch.com/javascript/react/get-window-size-with-react-hooks/
