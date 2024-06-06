import { Separator } from "@/components/ui/separator";
import { AccountSettingsForm } from "@/app/account/settings/components/account-settings-form";

export default function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">Settings</h3>
        <p className="text-sm text-muted-foreground">
          Manage your identity and password settings.
        </p>
      </div>
      <Separator />
      <AccountSettingsForm />
    </div>
  );
}
