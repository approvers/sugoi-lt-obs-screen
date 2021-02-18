export type Person = {
  userIcon?: string;
  identifier?: string;
  name: string;
}

export type Presentation = {
  presenter: Person;
  icon_fit_position?: number;
  title: string;
}

export type Service = "twitter" | "discord" | "youtube";

export type TimelineCard = {
  user: Person;
  service: Service;
  content: string;
}

export type ScreenData = {
  presentation: Presentation;
  timeline: TimelineCard[],
  notification?: string
}
