import { Presentation } from "../../../lib/data/ScreenData";
import styles from "../../../style/waiting/waiting_screen.module.scss";

type PresenterListElementProps = {
  presentation: Presentation;
};
export const PresenterListElement = ({
  presentation,
}: PresenterListElementProps): JSX.Element => (
  <div className={styles.list_element}>
    <p className={styles.list_element_presenter}>
      {presentation.presenter.name}
    </p>
    <p className={styles.list_element_title}>{presentation.title}</p>
  </div>
);
