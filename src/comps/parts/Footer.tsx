import * as React from "react";
import styles from "../../style/comps/footer.module.scss";
import {Presentation} from "../../lib/data/ScreenData";

type FooterProps = {
  presentation: Presentation
};

export const Footer: React.FC<FooterProps> = ({presentation}) => (
    <footer className={styles.footer_root}>
      <div className={styles.icon}>
        <img src={presentation.presenter.userIcon} style={{objectPosition: `0% ${(presentation.icon_fit_position ?? 0) * 100}%`}}/>

      </div>
      <p className={styles.presenter}>
        <span className={styles.presenter_name}>{presentation.presenter.name}</span>
        (
        <span className={styles.presenter_ident}>@{presentation.presenter.identifier}</span>
        )
      </p>
      <p className={styles.title}>
        {presentation.title}
      </p>
    </footer>
)
