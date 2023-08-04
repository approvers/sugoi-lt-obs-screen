import { ScreenData } from "../../lib/data/ScreenData";
import styles from "../../style/waiting/waiting_screen.module.scss";
import { PresenterList } from "./parts/PresenterList";
import { Title } from "./parts/Title";

type WaitingScreenProps = {
  state: ScreenData;
};
export const WaitingScreen = ({ state }: WaitingScreenProps): JSX.Element => (
  <div className={styles.wrapper}>
    <img className={styles.background} src={"/evil_spirits.png"} alt="" />
    <Title message={state.notification ?? "しばらくおまちください"} />
    <PresenterList presentations={state.pending_presentation} />
  </div>
);
