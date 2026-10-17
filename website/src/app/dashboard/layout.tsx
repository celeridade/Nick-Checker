import DashboardSidebar from "@/components/dashboard/DashboardSidebar";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-page">
      <DashboardSidebar />
      <div className="lg:pl-[260px]">
        <main className="mx-auto max-w-[1100px] px-4 py-6 sm:px-6">{children}</main>
      </div>
    </div>
  );
}
