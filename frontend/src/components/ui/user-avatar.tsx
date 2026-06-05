"use client";

import { Avatar, Indicator, type AvatarProps } from "@mantine/core";

type UserAvatarProps = AvatarProps & {
  email?: string;
  name: string;
  statusColor?: string;
};

const initials = (name: string) =>
  name
    .split(" ")
    .filter(Boolean)
    .map((part) => part[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();

const colors = ["cyan", "violet", "green", "yellow", "pink", "teal"];

const colorFor = (seed: string) => {
  const total = seed.split("").reduce((sum, char) => sum + char.charCodeAt(0), 0);
  return colors[total % colors.length];
};

export const UserAvatar = ({ email, name, statusColor, ...props }: UserAvatarProps) => {
  const avatar = (
    <Avatar color={colorFor(email ?? name)} radius="xl" {...props}>
      {initials(name)}
    </Avatar>
  );

  if (!statusColor) return avatar;

  return (
    <Indicator color={statusColor} size={9} offset={3} withBorder>
      {avatar}
    </Indicator>
  );
};
