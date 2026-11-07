// Real457590 dispatcher; movement/scans/formation leaves explicitly controlled.
// No original executable bytes or traces are embedded in this tool.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZFollowerPhaseOracle extends GhidraScript {
 static final long S=0x7fff0000L,U=0x5e30c8L,R=0x60ec00L,RET=0x12345000L;
 long get(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int s=0;s<1024;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{
   e.writeMemory(toAddr(U),new byte[384]);e.writeMemory(toAddr(R),new byte[144]);
   put(e,U+0x48,2,s%3==0?0:24);put(e,U+0x32,2,(s*31)%400-100);put(e,U+0x3c,1,s%5-2);put(e,U+0x8b,1,s%13);put(e,U+0x39,1,s%8);
   put(e,U+0x31,1,5+((s&2)!=0?8:0)+((s&4)!=0?128:0));put(e,U+0x64,1,(s&8)!=0?16:0);put(e,U+4,1,s%5==0?-1:20);put(e,U+2,1,s%128);
   put(e,R+0x3b,1,(s&16)!=0?9:255);put(e,R+0x3d,1,(s&32)!=0?2:0);put(e,R+0x13,1,1);put(e,U+192+0x5c,1,new int[]{5,-5,0}[s%3]);put(e,R+9,1,(s*17)&255);put(e,R+17,1,6);put(e,R+72+17,1,3);put(e,0x632194,4,R);put(e,0x622dc0,1,s&1);
   long[]calls={0x4502f0,0x460910,0x44a540,0x458500,0x416270,0x4574c0,0x44d3a0,0x4578b0,0x44d570,0x44cac0};for(long a:calls)e.setBreakpoint(toAddr(a));e.setBreakpoint(toAddr(RET));
   e.writeRegister("ESP",S);put(e,S,4,RET);put(e,S+4,4,U);e.writeRegister("ECX",0);if(!e.run(toAddr(0x457590),null,monitor))throw new Exception(e.getLastError());
   long hash=2166136261L;int count=0;
   while(e.readRegister("EIP").longValue()!=RET){if(++count>20)throw new Exception("call loop");long pc=e.readRegister("EIP").longValue();int c=0;while(c<calls.length&&calls[c]!=pc)c++;if(c==calls.length)throw new Exception("unexpected "+pc);hash=((hash^(c+1))*16777619)&0xffffffffL;
    long sp=e.readRegister("ESP").longValue();int choice=s%3==0?0xff00:0x900+s%14;
    switch(c){case 0:put(e,U+0x48,2,s%7==0?0:15);break;case 1:e.writeRegister("EAX",s*23);break;case 3:e.writeRegister("EAX",choice);break;case 4:e.writeRegister("EAX",s%4==0?1:0);break;case 5:put(e,U+0x31,1,get(e,U+0x31,1)|((s&64)!=0?8:0));break;case 6:put(e,get(e,sp+4,4),1,choice>>8);put(e,get(e,sp+8,4),1,choice&255);break;case 7:e.writeRegister("EAX",s%3==1?0:1);break;case 8:long group=R+((s&128)!=0?72:0);put(e,0x632194,4,group);put(e,group+9,1,162);put(e,U+0x32,2,77);put(e,U+0x3c,1,99);put(e,U+0x39,1,s*3&7);put(e,U+0x48,2,(s&256)!=0?0:12);put(e,U+0x31,1,get(e,U+0x31,1)|((s&512)!=0?128:0));break;default:break;}
    long ret=get(e,sp,4);e.writeRegister("ESP",sp+4);if(!e.run(toAddr(ret),null,monitor))throw new Exception(e.getLastError());
   }
   long group=get(e,0x632194,4);out.println("{\"scenario\":"+s+",\"values\":["+(short)get(e,U+0x48,2)+","+(short)get(e,U+0x32,2)+","+(byte)get(e,U+0x3c,1)+","+get(e,U+0x39,1)+","+get(e,U+0x31,1)+","+get(e,group+9,1)+","+hash+","+(group-R)+"]}");
  }finally{e.dispose();}
 }}}
}
