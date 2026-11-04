// Read effect labels and numeric IDs from the battle's declared effect table.
import ghidra.app.script.GhidraScript;
import java.io.*;
import java.nio.charset.StandardCharsets;
public class ZNativeSoundIds extends GhidraScript{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int bank=0;bank<5;bank++){long table=Integer.toUnsignedLong(getInt(toAddr(0x4a0dacL+bank*4)));for(int i=0;i<1024;i++){long a=table+i*44;int id=getShort(toAddr(a+2));if(id== -1)break;byte[]b=getBytes(toAddr(a+4),22);int length=0;while(length<b.length&&b[length]!=0)length++;out.println(bank+","+id+","+new String(b,0,length,StandardCharsets.US_ASCII)+","+getShort(toAddr(a+36))+","+getShort(toAddr(a))+","+getShort(toAddr(a+28)));}}}}
}
