export interface QuotaWindow { usedRatio: number; resetAt: string | null }
export interface QuotaSnapshot { week: QuotaWindow | null; fiveHour: QuotaWindow | null; plan: string | null; fetchedAt: string | null }
export interface AccountView { id: string; name: string; isCurrent: boolean; region: string; quota: QuotaSnapshot; status: string; error: string | null }
export interface AppSnapshot { accounts: AccountView[]; cliVersion: string | null; cliHome: string; activeState: string; error: string | null }
export interface LoginStatus { taskId: string; stage: string; verificationUrl: string | null; error: string | null }
export interface Api {
 snapshot(): Promise<AppSnapshot>;
 importCurrent(name: string, replaceId?: string): Promise<AppSnapshot>;
 rename(id: string, name: string): Promise<AppSnapshot>;
 remove(id: string): Promise<AppSnapshot>;
 switchAccount(id: string): Promise<AppSnapshot>;
 refresh(id?: string): Promise<AppSnapshot>;
 startLogin(region: string, name: string): Promise<LoginStatus>;
 loginStatus(taskId: string): Promise<LoginStatus>;
 cancelLogin(taskId: string): Promise<LoginStatus>;
}
