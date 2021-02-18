import * as React from "react";
import { TimelineCard } from "../../lib/data/ScreenData";
import styles from "../../style/comps/main.module.scss";
import { Mask } from "./main/Mask";
import { Timeline } from "./main/Timeline";

type MainProps = {
  timeline: TimelineCard[]
}
export const Main: React.FC<MainProps> = ({timeline}) => {
  return (
      <main className={styles.main_root}>
        <Mask />
        <Timeline timeline={timeline}/>
      </main>
  );
}
