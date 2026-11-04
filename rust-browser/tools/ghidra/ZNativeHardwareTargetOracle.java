// Actual45b050/44c830 callers. Target/spatial dependencies are recorded;
// all arithmetic,44c220 dispatch, muzzle helpers and44ab00 execute natively.
import ghidra.app.emulator.EmulatorHelper;import java.io.*;import java.nio.file.*;import java.util.*;import java.math.BigInteger;
public class ZNativeHardwareTargetOracle extends ZNativeWorldEffectOracle{
 static final long U=0x5e30c8,W=0x1900000,G=0x60ec00,P=0x1910000;
 int scenario,mode,resolves,rng,shotCount,engage,notice,finish,apc;long trace;
 long mix(long h,long v){return((h^(v&0xffffffffL))*16777619)&0xffffffffL;}
 int random(){return(scenario*7919^(rng++)*2027^0x7731a5c9);}
 int[] resolved(){return new int[]{560+(scenario/13%3)*5,612+(scenario/17%3)*7};}
 boolean fail(){return scenario%13==0||scenario%13==1&&resolves==2||scenario%13==2&&resolves==3||scenario%13==3&&resolves==4;}
 void invoke(EmulatorHelper e,long entry)throws Exception{
  resolves=rng=shotCount=engage=notice=finish=apc=0;trace=2166136261L;
  e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ECX",U);e.writeRegister("EDX",W);
  for(long a:new long[]{RET,0x449470,0x449bc0,0x44a3a0,0x4160c0,0x416270,0x458500,0x453ff0,0x458620,0x464e80,0x4648a0,0x464cf0,0x460910,0x447da0,0x45ace0,0x473264,0x41f4d0})e.setBreakpoint(toAddr(a));
  if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue(),sp=e.readRegister("ESP").longValue(),result=0;
   if(pc==0x449470){resolves++;int[]p=resolved();put(e,read(e,sp+8,4),2,p[0]);put(e,read(e,sp+12,4),2,p[1]);put(e,0x4d191c,4,scenario%3==0?0xffffffffL:U+192);put(e,0x6323ac,4,scenario%3==0?0xffffffffL:U+192);put(e,0x4d2be0,2,scenario%5==0?-1:p[0]+40);put(e,0x4d2be2,2,p[1]-40);result=fail()?1:0;}
   else if(pc==0x449bc0||pc==0x4160c0)engage++;
   else if(pc==0x44a3a0)notice++;
   else if(pc==0x416270)result=scenario%19==0?1:0;
   else if(pc==0x458500){put(e,U+119,1,read(e,U+119,1)+1&7);result=scenario%11==0?0xff00:0x100+(scenario%31);}
   else if(pc==0x453ff0||pc==0x458620)finish++;
   else if(pc==0x460910)result=random()&0xffffffffL;
   else if(pc==0x473264){double x=Double.longBitsToDouble(e.readRegister("XMM0").longValue());e.writeRegister("XMM0",BigInteger.valueOf(Double.doubleToRawLongBits(Math.sqrt(x))));}
   else if(pc==0x41f4d0)result=0;
   else if(pc==0x447da0){apc++;}
   else if(pc==0x45ace0)notice++;
   else if(pc==0x464e80||pc==0x4648a0||pc==0x464cf0){int kind=pc==0x4648a0?1:pc==0x464cf0?2:0;int[]v=new int[7];v[0]=kind;if(kind==1){for(int i=0;i<4;i++)v[i+1]=(short)read(e,sp+4+i*4,4);v[5]=(short)read(e,sp+20,4);v[6]=(int)read(e,sp+24,4);}else{v[1]=(short)e.readRegister("EDX").intValue();for(int i=0;i<3;i++)v[i+2]=(short)read(e,sp+4+i*4,4);v[5]=(short)read(e,sp+16,4);v[6]=kind==0?(byte)read(e,0x622be7,1):-1;}
    for(int n:v)trace=mix(trace,n);boolean bad=scenario%5==0||scenario%5==1&&shotCount==1;trace=mix(trace,bad?1:0);long p=P+shotCount*64;shotCount++;if(!bad&&kind!=2){put(e,read(e,sp+(kind==1?28:20),4),4,p);e.writeMemory(toAddr(p),new byte[64]);put(e,p+35,1,5);}result=bad?1:0;
   }else throw new Exception("Unexpected "+Long.toHexString(pc));
   e.writeRegister("EAX",result);long ret=read(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
  }
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  byte[]raw=Files.readAllBytes(Path.of("C:/GOG Games/Z/SINCOS.DAT"));for(int i=0;i<640;i++)put(e,TRIG+i*2,2,(short)(((raw[i*2]&255)<<8)|(raw[i*2+1]&255))>>5);put(e,0x586a80,4,TRIG);
  for(mode=0;mode<2;mode++)for(scenario=0;scenario<8192;scenario++){int s=scenario;e.writeMemory(toAddr(U),new byte[384]);e.writeMemory(toAddr(W),new byte[44]);e.writeMemory(toAddr(G),new byte[72]);e.writeMemory(toAddr(P),new byte[512]);put(e,U+8,4,W);put(e,0x632194,4,G);put(e,U+44,2,500);put(e,U+46,2,600);put(e,U+48,1,mode==0?new int[]{6,25,30,24,32,34,36,38}[s%8]:new int[]{5,21,22,23}[s%4]);put(e,U+124,1,mode==0?1:2);put(e,U+49,1,4+(s/4%2)*16);put(e,U+100,1,(s/8%2)*8);put(e,U+105,1,new int[]{0,32,128,160}[s/16%4]);put(e,U+57,1,s/64%8);put(e,U+60,1,new int[]{0,1,2,15,127,128,255}[s/512%7]);put(e,U+61,1,(s/43%3));put(e,U+123,1,new int[]{0,1,127,128,255}[s/7%5]);put(e,U+132,2,new int[]{-1,0,5,6,7,32767}[s/29%6]);put(e,U+119,1,s/11%8);put(e,U+2,1,s*17);put(e,U+4,1,s*19);put(e,U+120,1,s*23);put(e,U+121,1,s*13);put(e,U+142,1,s%3==0?3:15);put(e,U+158,1,s%23==0?2:0);put(e,W+14,2,560);put(e,W+16,2,612);put(e,W+18,1,new int[]{255,0,1,128,85}[s/31%5]);put(e,W+20,1,new int[]{0,1,127,128,255}[s/37%5]);put(e,W+21,1,s/64%8);put(e,W+5,1,new int[]{0,1,15,30,127,255}[s/41%6]);put(e,W+32,1,s*31);put(e,G+61,1,mode==0&&s%17==0?4:0);put(e,0x622dc0,1,s%2);put(e,0x622be7,1,77);put(e,0x4d191c,4,0xffffffffL);put(e,0x6323ac,4,0xffffffffL);put(e,0x4d2be0,2,0);put(e,U+192+44,2,560);put(e,U+192+46,2,612);
   invoke(e,mode==0?0x45b050:0x44c830);long tails=2166136261L;for(int i=0;i<shotCount;i++){tails=mix(tails,read(e,P+i*64+35,1));tails=mix(tails,read(e,P+i*64+37,1));}out.println("{\"mode\":"+mode+",\"scenario\":"+s+",\"timer\":"+(byte)read(e,U+60,1)+",\"timer3d\":"+read(e,U+61,1)+",\"reload\":"+(byte)read(e,U+123,1)+",\"counter84\":"+(short)read(e,U+132,2)+",\"counter14\":"+(byte)read(e,W+20,1)+",\"direction\":"+read(e,U+57,1)+",\"turret\":"+read(e,W+21,1)+",\"frame34\":"+read(e,U+52,1)+",\"flags31\":"+read(e,U+49,1)+",\"flags64\":"+read(e,U+100,1)+",\"flags69\":"+read(e,U+105,1)+",\"scan\":"+read(e,U+119,1)+",\"resolves\":"+resolves+",\"rng\":"+rng+",\"engage\":"+engage+",\"notice\":"+notice+",\"finish\":"+finish+",\"apc\":"+apc+",\"shots\":"+shotCount+",\"trace\":"+trace+",\"tails\":"+tails+"}");
  }
 }finally{e.dispose();}}}
}
