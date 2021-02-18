export type Person = {
  userIcon?: string;
  identifier?: string;
  name: string;
}

export type Presentation = {
  presenter: Person;
  title: string;
}

export type TimelineCard = {
  user: Person;
  service: "twitter" | "discord" | "youtube";
  content: string;
}

export type ScreenData = {
  presentation: Presentation;
  timeline: TimelineCard[],
  notification?: string
}
