export type Person = {
  userIcon?: string;
  identifier?: string;
  name: string;
};

export type Presentation = {
  presenter: Person;
  icon_fit_position?: number;
  title: string;
};

export type Service = "twitter" | "discord" | "youtube";

export type TimelineCard = {
  user: Person;
  service: Service;
  content: string;
};

export type Page = "LTScreen" | "WaitingScreen";

export type ScreenData = {
  presentation: Presentation;
  pending_presentation: Presentation[],
  timeline: TimelineCard[];
  notification?: string;
  transition: {
    current: Page,
    to?: Page
  }
};
