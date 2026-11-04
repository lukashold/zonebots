// Execute46e660/46e6d0/46f650 for every real battle descriptor. SDL playback
// is the explicit boundary and is tested with both success and failure.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeSampleOracle extends GhidraScript{
 static final long RET=0x12345000L,MIX=0x12346000L,SP=0x7fff0000L;
 static final int[] NOW={0,1,12,120,32767,-32768,-32768,0},LAST={0,0,0,0,32766,32767,0,-32768};int scenario;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 void sample(PrintWriter out,int bank,long table,int record,int clock,int mode,boolean enabled,boolean fort)throws Exception{
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{long entry=table+record*44;int id=getShort(toAddr(entry+2));int s=scenario++;put(e,0x6815a0,4,table);put(e,entry+28,2,LAST[clock]);put(e,0x680d08,2,NOW[clock]);put(e,0x6323a0,1,enabled?1:0);put(e,0x622dc2,1,s&1);put(e,0x622e18+(s&1),1,fort?255:2);put(e,0x681588,2,s*17);put(e,0x681580,2,s*29);put(e,0x474214,4,MIX);
   e.setBreakpoint(toAddr(RET));e.setBreakpoint(toAddr(MIX));e.writeRegister("ESP",SP);put(e,SP,4,RET);e.writeRegister("ECX",id);e.writeRegister("EAX",0x11223344);long function=new long[]{0x46e660,0x46e6d0,0x46f650}[mode];if(!e.run(toAddr(function),null,monitor))throw new Exception(e.getLastError());boolean played=e.readRegister("EIP").longValue()==MIX;int mixer=s%2==0?-1:3;
   if(played){e.writeRegister("EAX",mixer);long stack=e.readRegister("ESP").longValue(),next=read(e,stack,4);e.writeRegister("ESP",stack+4);if(!e.run(toAddr(next),null,monitor))throw new Exception(e.getLastError());}
   out.println(s+","+bank+","+record+","+clock+","+mode+","+(enabled?1:0)+","+(fort?1:0)+","+(mode==0?e.readRegister("EAX").intValue():-1)+","+(played?1:0)+","+(short)read(e,entry+28,2)+","+read(e,0x681588,2)+","+read(e,0x681580,2)+","+mixer);
  }finally{e.dispose();}
 }
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int bank=0;bank<5;bank++){long table=Integer.toUnsignedLong(getInt(toAddr(0x4a0dacL+bank*4)));int notice=-1;for(int i=0;i<1024;i++){int id=getShort(toAddr(table+i*44+2));if(id== -1)break;if(id==196)notice=i;for(int clock=0;clock<8;clock++)for(int mode=0;mode<2;mode++)sample(out,bank,table,i,clock,mode,(i+clock)%3!=0,true);}if(notice<0)throw new Exception("No196 notice descriptor");for(int clock=0;clock<8;clock++)for(int enabled=0;enabled<2;enabled++)for(int fort=0;fort<2;fort++)sample(out,bank,table,notice,clock,2,enabled!=0,fort!=0);}}}
}
