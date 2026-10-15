import { describe,it,expect } from 'vitest';
import {formatQuota,formatReset} from '../components/quota-bar';
describe('quota formatting',()=>{
 it('distinguishes missing data from zero',()=>{expect(formatQuota(null)).toBe('暂无数据');expect(formatQuota(0)).toBe('0%');expect(formatQuota(0.3)).toBe('30%');expect(formatQuota(1.2)).toBe('120%');});
 it('handles invalid dates and local cross-day reset',()=>{expect(formatReset('bad')).toBe('重置时间未知');expect(formatReset('2026-10-14T20:00:00Z','zh-CN','Asia/Shanghai')).toContain('10-15 04:00');});
});
