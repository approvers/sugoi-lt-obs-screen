import * as React from "react";
import styles from "../../style/comps/footer.module.scss";
import {Presentation} from "../../lib/data/ScreenData";

type FooterProps = {
  presentation: Presentation
};

export const Footer: React.FC<FooterProps> = ({presentation}) => (
    <footer className={styles.footer_root}>
      <p>
        <span>{presentation.presenter.name}</span>
        <span>{presentation.presenter.identifier}</span>
        (Icon: {presentation.presenter.userIcon})
      </p>
      <p>
        {presentation.title}
      </p>
    </footer>
)
