import * as React from "react";
import styles from "../../style/comps/main.module.scss";
import { Mask } from "./main/Mask";
import { Timeline } from "./main/Timeline";

export const Main = () => {
  return (
      <main className={styles.main_root}>
        <Mask />
        <Timeline />
      </main>
  );
}
