import styles from "../../../style/ltscreen/notification.module.scss";

type NotificationProps = {
  notification: string;
};
export const Notification = ({ notification }: NotificationProps): JSX.Element => (
  <header className={styles.notification_root}>{notification}</header>
);
