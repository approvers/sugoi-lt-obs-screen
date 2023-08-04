import { useWindowDimensions } from "../../../../lib/WindowHooks";
import styles from "../../../../style/ltscreen/main.module.scss";

export const Mask = (): JSX.Element => {
  const { width } = useWindowDimensions();

  // TODO: この数値をどうにかしたい
  const netWidth = width * 0.8075;

  return (
    <div
      className={styles.mask}
      style={{
        width: `${netWidth}px`,
        minWidth: `${netWidth}px`,
        maxWidth: `${netWidth}px`,
        height: `${netWidth * (9 / 16)}px`,
      }}
    />
  );
};
