import * as React from "react";

import styles from "../../../style/ltscreen/notification.module.scss";

type NotificationProps = {
  notification: string;
};
export const Notification: React.FC<NotificationProps> = ({ notification }) => (
  <header className={styles.notification_root}>{notification}</header>
);
