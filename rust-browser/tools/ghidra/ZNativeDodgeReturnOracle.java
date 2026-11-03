// Executes replacement movement lines and their real end/block return chain.
// The primary path is an independent native node; collision is an input gate.
import ghidra.app.emulator.EmulatorHelper;
import java.io.PrintWriter;
public class ZNativeDodgeReturnOracle extends ZNativeReactionsOracle{
 static final long H=0x612f80L,N=0x5eee28L;
 int collision,stepTick,contacts;
 void invoke(EmulatorHelper e,long entry,long ecx,long edx,long...args)throws Exception{
  e.writeRegister("ESP",S);put(e,S,4,RET);for(int i=0;i<args.length;i++)put(e,S+4+i*4,4,args[i]);e.writeRegister("ECX",ecx);e.writeRegister("EDX",edx);e.setBreakpoint(toAddr(RET));
  if(!e.run(toAddr(entry),null,monitor))throw new Exception(e.getLastError());
  while(e.readRegister("EIP").longValue()!=RET){long pc=e.readRegister("EIP").longValue();
   if(pc==0x45a5c0)e.writeRegister("EAX",collision);
   else if(pc==0x451330)e.writeRegister("EAX",0);
   else if(pc==0x450db0){throw new Exception("Unexpected contact breakpoint");}
   else if(pc==0x450890)e.writeRegister("EAX",1);
   else if(pc==0x44d570){contacts++;}
   else throw new Exception("Unexpected "+Long.toHexString(pc));
   long stack=e.readRegister("ESP").longValue(),ret=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
  }
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{
  for(long a:new long[]{0x45a5c0,0x451330,0x450890,0x44d570})e.setBreakpoint(toAddr(a));
  int scenario=0;for(int cls:new int[]{0,1})for(int path=0;path<2;path++)for(int blocked=0;blocked<2;blocked++)for(int speed=1;speed<=4;speed++)for(int[]d:new int[][]{{9,3},{3,9},{-9,3},{-3,9},{9,-3},{3,-9},{-9,-3},{-3,-9}}){
   e.writeMemory(toAddr(U),new byte[192]);e.writeMemory(toAddr(H),new byte[44]);e.writeMemory(toAddr(G),new byte[72]);e.writeMemory(toAddr(N),new byte[8]);put(e,0x632194,4,G);put(e,0x622dc0,1,0);put(e,U+8,4,H);put(e,U+0x2c,2,500);put(e,U+0x2e,2,500);put(e,U+0x31,1,4);put(e,U+0x35,1,cls==1?9:1);put(e,U+0x5e,1,1);put(e,U+0x64,1,64);put(e,U+0x8c,1,2);put(e,U+0x7c,1,cls);put(e,U+0x8b,1,7);put(e,U+0x62,1,speed);put(e,U+0x34,1,255);if(path!=0){put(e,U+0x14,4,N);put(e,N,2,760);put(e,N+2,2,652);}contacts=0;
   invoke(e,0x4525a0,U,500+d[0],500+d[1]);
   out.println("{\"kind\":\"begin\",\"scenario\":"+scenario+",\"state\":"+read(e,U+0x35,1)+",\"flags\":"+read(e,U+0x64,1)+",\"path\":"+(read(e,U+0x14,4)==N?1:0)+",\"remaining\":"+(short)read(e,U+0x48,2)+",\"direction\":"+read(e,U+0x39,1)+",\"hardware\":"+read(e,H+0x17,1)+"}");
   for(stepTick=1;stepTick<=16;stepTick++){
    collision=blocked!=0&&stepTick==2?1:0;put(e,0x635b64,2,read(e,U+0x2c,2));put(e,0x635b68,2,read(e,U+0x2e,2));put(e,0x635b60,4,0x45a5c0);int axis=(int)read(e,U+0x81,1);long entry=axis==0?0x4520a0:axis==2?0x451fa0:axis==4?0x451da0:0x451ea0;
    invoke(e,entry,U,P,P+1);put(e,U+0x2c,2,read(e,0x635b64,2));put(e,U+0x2e,2,read(e,0x635b68,2));
    if(cls==0&&read(e,U+0x35,1)!=2)invoke(e,0x450db0,U,0);
    out.println("{\"kind\":\"step\",\"scenario\":"+scenario+",\"tick\":"+stepTick+",\"x\":"+(short)read(e,U+0x2c,2)+",\"y\":"+(short)read(e,U+0x2e,2)+",\"state\":"+read(e,U+0x35,1)+",\"flags\":"+read(e,U+0x64,1)+",\"path\":"+(read(e,U+0x14,4)==N?1:0)+",\"remaining\":"+(short)read(e,U+0x48,2)+",\"hardware\":"+read(e,H+0x17,1)+"}");
    if((read(e,U+0x64,1)&64)==0)break;
   }scenario++;
  }
 }finally{e.dispose();}}}
}
