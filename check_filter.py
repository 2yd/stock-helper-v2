import json, sys, urllib.request

url = "http://82.push2.eastmoney.com/api/qt/clist/get?pn=1&pz=5000&po=1&np=1&ut=bd1d9ddb04089700cf9c27f6f7426281&fltt=2&invt=2&fid=f3&fs=m:0+t:6,m:0+t:80,m:1+t:2&fields=f2,f3,f6,f9,f12,f14,f20,f23,f37"
data = json.loads(urllib.request.urlopen(url).read())
items = data['data']['diff']
total = len(items)
print(f'Total stocks: {total}')

passed = 0
fail_reasons = {}
dash_fields = {'f2':0,'f9':0,'f23':0,'f37':0,'f6':0,'f20':0}

for i in items:
    # Check for "-" string values
    for fk in dash_fields:
        if isinstance(i.get(fk), str):
            dash_fields[fk] += 1
    
    price = i.get('f2', 0)
    pe = i.get('f9', 0)
    pb = i.get('f23', 0)
    roe = i.get('f37', 0)
    cap = i.get('f20', 0)
    amount = i.get('f6', 0)
    name = i.get('f14', '')
    
    if isinstance(price, str): price = 0
    if isinstance(pe, str): pe = 0
    if isinstance(pb, str): pb = 0
    if isinstance(roe, str): roe = 0
    if isinstance(cap, str): cap = 0
    if isinstance(amount, str): amount = 0

    reasons = []
    if 'ST' in name or 'st' in name:
        reasons.append('ST')
    if price <= 0 or amount <= 0:
        reasons.append('停牌')
    if price < 3:
        reasons.append('价格<3')
    if cap / 1e8 < 30:
        reasons.append('市值<30亿')
    if amount / 1e4 < 5000:
        reasons.append('成交额<5000万')
    if pe > 100:
        reasons.append('PE>100')
    if 0 < pe < 0:  # won't hit, fix below
        pass
    if pe < 0:
        reasons.append('PE<0(亏损)')
    if pe == 0:
        reasons.append('PE=0')
    if pb > 20:
        reasons.append('PB>20')
    if roe < 5:
        reasons.append('ROE<5%')
    
    if not reasons:
        passed += 1
    else:
        for r in reasons:
            fail_reasons[r] = fail_reasons.get(r, 0) + 1

print(f'\nPassed all filters: {passed}')
print(f'\nDash/string values per field:')
for k, v in dash_fields.items():
    print(f'  {k}: {v} strings out of {total}')
print(f'\nFail reasons (a stock can fail multiple):')
for k, v in sorted(fail_reasons.items(), key=lambda x: -x[1]):
    print(f'  {k}: {v}')
