import { TimelineCard } from "../../../lib/data/ScreenData";
import styles from "../../../style/ltscreen/main.module.scss";
import { Mask } from "./main/Mask";
import { Timeline } from "./main/Timeline";

type MainProps = {
  timeline: Array<TimelineCard>;
};
export const Main = ({ timeline }: MainProps): JSX.Element => {
  return (
    <main className={styles.main_root}>
      <Mask />
      <Timeline timeline={timeline} />
    </main>
  );
};
