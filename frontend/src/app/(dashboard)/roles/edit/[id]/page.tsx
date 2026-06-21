"use client";

import { RoleEditor } from "@components/admin/role-editor";
import { useParams } from "next/navigation";

export default function EditRolePage() {
  const params = useParams<{ id: string }>();
  return <RoleEditor id={params?.id} />;
}
