import * as React from "react";
import styles from "../../../style/comps/main.module.scss";
import {useWindowDimensions} from "../../../lib/WindowHooks";

export const Mask = () => {
  const {width} = useWindowDimensions();
  const netWidth = width * 0.8;

  return (
      <div
        className={styles.mask}
        style={{
          width: `${netWidth}px`,
          height: `${netWidth * (9 / 16)}px`, }}
      />
  )
}
