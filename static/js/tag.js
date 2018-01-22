function Tag(inputId){
	var obj = new Object();
	if(inputId==null||inputId==""){
		alert("初始化失败，请检查参数！");
		return;
	}
	obj.inputId = inputId;
	//初始化
	obj = (function(obj){
		obj.tagValue="";
		obj.isSearchable = false;
		return obj;
	})(obj);
	
	//初始化界面
	obj.initView=function(){
		var inputObj = $("#"+this.inputId);
		var inputId = this.inputId;
		inputObj.css("display","none");
		var appendStr='';
		appendStr+='<div class="tagsContaine" id="'+inputId+'_tagcontaine">';
		appendStr+='<div class="tagList"></div>';
		appendStr+='</div>';
		inputObj.after(appendStr);
		var tagInput = $("#"+inputId+"_tagcontaine .tagInput");
		if(this.tagValue!=null&&this.tagValue!=""){
			tagTake.setTags(inputId,this.tagValue,this.isSearchable);
		}
	}
	
	return obj;
}

var tagTake ={
	"setTags":function(inputId,inputValue,isSearch){
		if(inputValue==null||inputValue==""){
			return;
		}
		var tagListContaine = $("#"+inputId+"_tagcontaine .tagList");
		inputValue = inputValue.replace(/，/g,",");
		var inputValueArray = inputValue.split(",");
		for(var i=0;i<inputValueArray.length;i++){
			var valueItem = $.trim(inputValueArray[i]);
			if(valueItem!=""){
				var appendListItem = tagTake.getTagItemModel(valueItem,isSearch);
				tagListContaine.append(appendListItem);
			}
		}
	},
	"getTagItemModel":function(valueStr,isSearch){
		if(isSearch){
			return '<div class="news-label" ><a href ="#">'+valueStr+'</a></div>';
		} else {
			return '<div class="news-label"><span>'+valueStr+'</span></div>';
		}
	}
}

